# AlphaForge Message Bus (Python fallback)
"""
Message bus implementation for AlphaForge trading system.
This is the Python fallback when Rust extensions are not available.
"""

import asyncio
import threading
from typing import Any, Dict, List, Callable, Optional, Set, TypeVar, Generic
from dataclasses import dataclass
from enum import Enum
from collections import defaultdict
from alphaforge.core.uuid import uuid4_new


T = TypeVar('T')
MessageHandler = Callable[[Any], None]
AsyncMessageHandler = Callable[[Any], asyncio.Future]


class MessageType(Enum):
    """Message type enumeration."""
    PUBLISH = "publish"
    REQUEST = "request"
    RESPONSE = "response"
    POINT_TO_POINT = "p2p"


@dataclass
class Message:
    """Message wrapper."""
    id: str
    type: MessageType
    topic: str
    payload: Any
    correlation_id: Optional[str] = None
    reply_to: Optional[str] = None
    timestamp: float = 0.0
    
    def __post_init__(self):
        if self.timestamp == 0.0:
            import time
            self.timestamp = time.time()


class MessageBus:
    """High-performance message bus for event-driven architecture."""
    
    def __init__(self):
        self._subscribers: Dict[str, List[MessageHandler]] = defaultdict(list)
        self._async_subscribers: Dict[str, List[AsyncMessageHandler]] = defaultdict(list)
        self._request_handlers: Dict[str, MessageHandler] = {}
        self._async_request_handlers: Dict[str, AsyncMessageHandler] = {}
        self._pending_requests: Dict[str, asyncio.Future] = {}
        self._lock = threading.RLock()
        self._running = False
        self._message_queue: asyncio.Queue = None
        self._worker_task: Optional[asyncio.Task] = None
    
    async def start(self) -> None:
        """Start the message bus."""
        if self._running:
            return
        
        self._running = True
        self._message_queue = asyncio.Queue(maxsize=100000)  # High capacity queue
        self._worker_task = asyncio.create_task(self._message_worker())
    
    async def stop(self) -> None:
        """Stop the message bus."""
        if not self._running:
            return
        
        self._running = False
        
        if self._worker_task:
            self._worker_task.cancel()
            try:
                await self._worker_task
            except asyncio.CancelledError:
                pass
        
        # Complete pending requests
        with self._lock:
            for future in self._pending_requests.values():
                if not future.done():
                    future.cancel()
            self._pending_requests.clear()
    
    def subscribe(self, topic: str, handler: MessageHandler) -> str:
        """
        Subscribe to a topic with synchronous handler.
        
        Returns:
            Subscription ID for unsubscribing
        """
        with self._lock:
            subscription_id = uuid4_new()
            self._subscribers[topic].append(handler)
            return subscription_id
    
    def subscribe_async(self, topic: str, handler: AsyncMessageHandler) -> str:
        """
        Subscribe to a topic with asynchronous handler.
        
        Returns:
            Subscription ID for unsubscribing
        """
        with self._lock:
            subscription_id = uuid4_new()
            self._async_subscribers[topic].append(handler)
            return subscription_id
    
    def unsubscribe(self, topic: str, subscription_id: str) -> bool:
        """
        Unsubscribe from a topic.
        
        Returns:
            True if unsubscribed successfully
        """
        with self._lock:
            # Note: In a full implementation, we'd track subscription IDs
            # For now, remove all handlers for the topic
            if topic in self._subscribers:
                self._subscribers[topic].clear()
                return True
            if topic in self._async_subscribers:
                self._async_subscribers[topic].clear()
                return True
            return False
    
    async def publish(self, topic: str, message: Any) -> None:
        """Publish a message to all subscribers of a topic."""
        if not self._running or not self._message_queue:
            return
        
        msg = Message(
            id=uuid4_new(),
            type=MessageType.PUBLISH,
            topic=topic,
            payload=message
        )
        
        try:
            await self._message_queue.put(msg)
        except asyncio.QueueFull:
            # In production, this would be logged as a warning
            pass
    
    def publish_sync(self, topic: str, message: Any) -> None:
        """Publish a message synchronously (for sync contexts)."""
        # Direct dispatch to sync handlers only
        with self._lock:
            handlers = self._subscribers.get(topic, [])
            for handler in handlers:
                try:
                    handler(message)
                except Exception:
                    # Log error in production
                    pass
    
    async def request(self, topic: str, message: Any, timeout: float = 5.0) -> Any:
        """
        Send a request and wait for response.
        
        Returns:
            Response payload
            
        Raises:
            asyncio.TimeoutError: If no response within timeout
        """
        if not self._running or not self._message_queue:
            raise RuntimeError("Message bus not running")
        
        correlation_id = uuid4_new()
        reply_to = f"_reply_{correlation_id}"
        
        # Create response future
        response_future = asyncio.Future()
        with self._lock:
            self._pending_requests[correlation_id] = response_future
        
        # Send request
        msg = Message(
            id=uuid4_new(),
            type=MessageType.REQUEST,
            topic=topic,
            payload=message,
            correlation_id=correlation_id,
            reply_to=reply_to
        )
        
        try:
            await self._message_queue.put(msg)
            return await asyncio.wait_for(response_future, timeout=timeout)
        except asyncio.TimeoutError:
            with self._lock:
                self._pending_requests.pop(correlation_id, None)
            raise
    
    async def send_response(self, request_msg: Message, response: Any) -> None:
        """Send a response to a request."""
        if not request_msg.reply_to or not request_msg.correlation_id:
            return
        
        msg = Message(
            id=uuid4_new(),
            type=MessageType.RESPONSE,
            topic=request_msg.reply_to,
            payload=response,
            correlation_id=request_msg.correlation_id
        )
        
        if self._message_queue:
            try:
                await self._message_queue.put(msg)
            except asyncio.QueueFull:
                pass
    
    def register_request_handler(self, topic: str, handler: MessageHandler) -> None:
        """Register a synchronous request handler."""
        with self._lock:
            self._request_handlers[topic] = handler
    
    def register_async_request_handler(self, topic: str, handler: AsyncMessageHandler) -> None:
        """Register an asynchronous request handler."""
        with self._lock:
            self._async_request_handlers[topic] = handler
    
    async def send_direct(self, endpoint: str, message: Any) -> None:
        """Send a point-to-point message."""
        msg = Message(
            id=uuid4_new(),
            type=MessageType.POINT_TO_POINT,
            topic=endpoint,
            payload=message
        )
        
        if self._message_queue:
            try:
                await self._message_queue.put(msg)
            except asyncio.QueueFull:
                pass
    
    async def _message_worker(self) -> None:
        """Background worker to process messages."""
        while self._running:
            try:
                # Use timeout to allow checking _running periodically
                message = await asyncio.wait_for(
                    self._message_queue.get(), timeout=0.1
                )
                await self._process_message(message)
            except asyncio.TimeoutError:
                continue
            except asyncio.CancelledError:
                break
            except Exception:
                # Log error in production
                continue
    
    async def _process_message(self, message: Message) -> None:
        """Process a single message."""
        try:
            if message.type == MessageType.PUBLISH:
                await self._handle_publish(message)
            elif message.type == MessageType.REQUEST:
                await self._handle_request(message)
            elif message.type == MessageType.RESPONSE:
                await self._handle_response(message)
            elif message.type == MessageType.POINT_TO_POINT:
                await self._handle_p2p(message)
        except Exception:
            # Log error in production
            pass
    
    async def _handle_publish(self, message: Message) -> None:
        """Handle publish message."""
        # Sync handlers
        with self._lock:
            sync_handlers = self._subscribers.get(message.topic, []).copy()
            async_handlers = self._async_subscribers.get(message.topic, []).copy()
        
        # Execute sync handlers
        for handler in sync_handlers:
            try:
                handler(message.payload)
            except Exception:
                # Log error in production
                pass
        
        # Execute async handlers
        tasks = []
        for handler in async_handlers:
            try:
                task = asyncio.create_task(handler(message.payload))
                tasks.append(task)
            except Exception:
                # Log error in production
                pass
        
        # Wait for async handlers (with timeout to prevent blocking)
        if tasks:
            try:
                await asyncio.wait_for(asyncio.gather(*tasks, return_exceptions=True), timeout=1.0)
            except asyncio.TimeoutError:
                # Log warning in production
                pass
    
    async def _handle_request(self, message: Message) -> None:
        """Handle request message."""
        response = None
        
        with self._lock:
            sync_handler = self._request_handlers.get(message.topic)
            async_handler = self._async_request_handlers.get(message.topic)
        
        try:
            if async_handler:
                response = await async_handler(message.payload)
            elif sync_handler:
                response = sync_handler(message.payload)
            
            if response is not None:
                await self.send_response(message, response)
        except Exception:
            # Log error and send error response in production
            await self.send_response(message, {"error": "Request handler failed"})
    
    async def _handle_response(self, message: Message) -> None:
        """Handle response message."""
        if not message.correlation_id:
            return
        
        with self._lock:
            future = self._pending_requests.pop(message.correlation_id, None)
        
        if future and not future.done():
            future.set_result(message.payload)
    
    async def _handle_p2p(self, message: Message) -> None:
        """Handle point-to-point message."""
        # Same as publish but typically for specific endpoints
        await self._handle_publish(message)
    
    @property
    def is_running(self) -> bool:
        """Check if message bus is running."""
        return self._running
    
    def get_stats(self) -> Dict[str, Any]:
        """Get message bus statistics."""
        with self._lock:
            return {
                "running": self._running,
                "total_topics": len(self._subscribers) + len(self._async_subscribers),
                "sync_subscribers": sum(len(handlers) for handlers in self._subscribers.values()),
                "async_subscribers": sum(len(handlers) for handlers in self._async_subscribers.values()),
                "request_handlers": len(self._request_handlers) + len(self._async_request_handlers),
                "pending_requests": len(self._pending_requests),
                "queue_size": self._message_queue.qsize() if self._message_queue else 0
            }
