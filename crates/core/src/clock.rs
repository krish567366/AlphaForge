//! High-performance clock abstractions for AlphaForge

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::time::{UnixNanos, unix_nanos_now};
use crate::error::{AlphaForgeError, Result};

/// Timer callback function type
pub type TimerCallback = Box<dyn Fn() + Send + Sync>;

/// Timer information
#[derive(Debug, Clone)]
pub struct Timer {
    pub name: String,
    pub interval_ns: u64,
    pub next_time_ns: u64,
    pub stop_time_ns: Option<u64>,
    pub callback: Arc<dyn Fn() + Send + Sync>,
}

/// Clock abstraction for unified time handling
#[async_trait]
pub trait Clock: Send + Sync {
    /// Get current timestamp in nanoseconds
    fn timestamp_ns(&self) -> UnixNanos;
    
    /// Set a timer with callback
    async fn set_timer(
        &mut self,
        name: String,
        interval_ns: u64,
        start_time_ns: u64,
        stop_time_ns: Option<u64>,
        callback: TimerCallback,
    ) -> Result<()>;
    
    /// Cancel a timer
    async fn cancel_timer(&mut self, name: String) -> Result<()>;
    
    /// Get next scheduled timer time
    fn next_timer_ns(&self) -> Option<UnixNanos>;
}

/// Live clock implementation using system time
pub struct LiveClock {
    timers: Arc<Mutex<HashMap<String, Timer>>>,
    timer_tx: mpsc::UnboundedSender<TimerCommand>,
}

#[derive(Debug)]
enum TimerCommand {
    Set {
        name: String,
        interval_ns: u64,
        start_time_ns: u64,
        stop_time_ns: Option<u64>,
        callback: Arc<dyn Fn() + Send + Sync>,
    },
    Cancel {
        name: String,
    },
}

impl LiveClock {
    /// Create a new live clock
    pub fn new() -> Self {
        let (timer_tx, mut timer_rx) = mpsc::unbounded_channel();
        let timers = Arc::new(Mutex::new(HashMap::new()));
        
        // Spawn timer management task
        let timers_clone = Arc::clone(&timers);
        tokio::spawn(async move {
            let mut active_timers: HashMap<String, Timer> = HashMap::new();
            
            loop {
                tokio::select! {
                    // Handle timer commands
                    cmd = timer_rx.recv() => {
                        match cmd {
                            Some(TimerCommand::Set { name, interval_ns, start_time_ns, stop_time_ns, callback }) => {
                                let timer = Timer {
                                    name: name.clone(),
                                    interval_ns,
                                    next_time_ns: start_time_ns,
                                    stop_time_ns,
                                    callback,
                                };
                                active_timers.insert(name, timer);
                                debug!("Timer set: {}", timer.name);
                            }
                            Some(TimerCommand::Cancel { name }) => {
                                active_timers.remove(&name);
                                debug!("Timer cancelled: {}", name);
                            }
                            None => break, // Channel closed
                        }
                    }
                    
                    // Check for timer expiration
                    _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => {
                        let now = unix_nanos_now();
                        let mut expired_timers = Vec::new();
                        
                        for (name, timer) in &mut active_timers {
                            if now >= timer.next_time_ns {
                                // Timer expired, execute callback
                                (timer.callback)();
                                
                                // Check if timer should continue
                                if let Some(stop_time) = timer.stop_time_ns {
                                    if now >= stop_time {
                                        expired_timers.push(name.clone());
                                        continue;
                                    }
                                }
                                
                                // Schedule next execution
                                timer.next_time_ns = now + timer.interval_ns;
                            }
                        }
                        
                        // Remove expired timers
                        for name in expired_timers {
                            active_timers.remove(&name);
                            debug!("Timer expired and removed: {}", name);
                        }
                    }
                }
            }
        });
        
        Self {
            timers,
            timer_tx,
        }
    }
}

#[async_trait]
impl Clock for LiveClock {
    fn timestamp_ns(&self) -> UnixNanos {
        unix_nanos_now()
    }
    
    async fn set_timer(
        &mut self,
        name: String,
        interval_ns: u64,
        start_time_ns: u64,
        stop_time_ns: Option<u64>,
        callback: TimerCallback,
    ) -> Result<()> {
        let cmd = TimerCommand::Set {
            name,
            interval_ns,
            start_time_ns,
            stop_time_ns,
            callback: Arc::from(callback),
        };
        
        self.timer_tx.send(cmd)
            .map_err(|_| AlphaForgeError::Component { 
                msg: "Timer system unavailable".to_string()
            })?;
            
        Ok(())
    }
    
    async fn cancel_timer(&mut self, name: String) -> Result<()> {
        let cmd = TimerCommand::Cancel { name };
        
        self.timer_tx.send(cmd)
            .map_err(|_| AlphaForgeError::Component { 
                msg: "Timer system unavailable".to_string()
            })?;
            
        Ok(())
    }
    
    fn next_timer_ns(&self) -> Option<UnixNanos> {
        // For live clock, always return current time + small buffer
        Some(unix_nanos_now() + 1_000_000) // 1ms buffer
    }
}

impl Default for LiveClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Test clock for backtesting with controllable time
pub struct TestClock {
    current_time: std::sync::atomic::AtomicU64,
    timers: Arc<Mutex<HashMap<String, Timer>>>,
}

impl TestClock {
    /// Create a new test clock with specified start time
    pub fn new(start_time_ns: UnixNanos) -> Self {
        Self {
            current_time: std::sync::atomic::AtomicU64::new(start_time_ns),
            timers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Advance time by specified duration
    pub async fn advance_time(&self, duration_ns: u64) {
        let current = self.current_time.load(std::sync::atomic::Ordering::Relaxed);
        let new_time = current + duration_ns;
        self.current_time.store(new_time, std::sync::atomic::Ordering::Relaxed);
        
        // Process expired timers
        let timers = self.timers.lock().await;
        for timer in timers.values() {
            if new_time >= timer.next_time_ns {
                (timer.callback)();
            }
        }
    }
    
    /// Set time to specific timestamp
    pub fn set_time(&self, timestamp_ns: UnixNanos) {
        self.current_time.store(timestamp_ns, std::sync::atomic::Ordering::Relaxed);
    }
}

#[async_trait]
impl Clock for TestClock {
    fn timestamp_ns(&self) -> UnixNanos {
        self.current_time.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    async fn set_timer(
        &mut self,
        name: String,
        interval_ns: u64,
        start_time_ns: u64,
        stop_time_ns: Option<u64>,
        callback: TimerCallback,
    ) -> Result<()> {
        let timer = Timer {
            name: name.clone(),
            interval_ns,
            next_time_ns: start_time_ns,
            stop_time_ns,
            callback: Arc::from(callback),
        };
        
        self.timers.lock().await.insert(name, timer);
        Ok(())
    }
    
    async fn cancel_timer(&mut self, name: String) -> Result<()> {
        self.timers.lock().await.remove(&name);
        Ok(())
    }
    
    fn next_timer_ns(&self) -> Option<UnixNanos> {
        // For test clock, return earliest timer
        self.current_time.load(std::sync::atomic::Ordering::Relaxed).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_live_clock_basic() {
        let clock = LiveClock::new();
        let now = clock.timestamp_ns();
        
        sleep(Duration::from_millis(1)).await;
        
        let later = clock.timestamp_ns();
        assert!(later > now);
    }
    
    #[tokio::test]
    async fn test_live_clock_timer() {
        let mut clock = LiveClock::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);
        
        let start_time = clock.timestamp_ns() + 10_000_000; // 10ms from now
        
        clock.set_timer(
            "test_timer".to_string(),
            1_000_000, // 1ms interval
            start_time,
            None,
            Box::new(move || {
                called_clone.store(true, Ordering::Relaxed);
            }),
        ).await.unwrap();
        
        // Wait for timer to fire
        sleep(Duration::from_millis(20)).await;
        
        assert!(called.load(Ordering::Relaxed));
    }
    
    #[test]
    fn test_test_clock() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            let start_time = 1000000000000000000; // Some fixed time
            let clock = TestClock::new(start_time);
            
            assert_eq!(clock.timestamp_ns(), start_time);
            
            clock.advance_time(1000000000).await; // 1 second
            assert_eq!(clock.timestamp_ns(), start_time + 1000000000);
        });
    }
}
