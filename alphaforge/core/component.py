# AlphaForge Component System
"""
Base component class with finite state machine lifecycle management.
"""

import asyncio
from abc import ABC, abstractmethod
from enum import Enum
from typing import Optional, Dict, Any
import logging
from alphaforge.core.exceptions import ComponentError


class ComponentState(Enum):
    """Component lifecycle states."""
    INITIALIZING = "INITIALIZING"
    INITIALIZED = "INITIALIZED"
    STARTING = "STARTING"
    RUNNING = "RUNNING"
    STOPPING = "STOPPING"
    STOPPED = "STOPPED"
    RESUMING = "RESUMING" 
    ERROR = "ERROR"
    DISPOSED = "DISPOSED"


class Component(ABC):
    """
    Base class for all AlphaForge system components.
    
    Provides finite state machine lifecycle management with async support.
    All components must implement the abstract lifecycle methods.
    """
    
    def __init__(self, name: str, config: Optional[Dict[str, Any]] = None):
        self.name = name
        self.config = config or {}
        self._state = ComponentState.INITIALIZING
        self._logger = logging.getLogger(f"alphaforge.{name}")
        
        # State transition tracking
        self._state_history: list[ComponentState] = [ComponentState.INITIALIZING]
        
    @property
    def state(self) -> ComponentState:
        """Get current component state."""
        return self._state
        
    @property
    def is_initialized(self) -> bool:
        """Check if component is initialized."""
        return self._state in (
            ComponentState.INITIALIZED,
            ComponentState.STARTING,
            ComponentState.RUNNING,
            ComponentState.STOPPING,
            ComponentState.STOPPED,
            ComponentState.RESUMING,
        )
        
    @property
    def is_running(self) -> bool:
        """Check if component is running."""
        return self._state == ComponentState.RUNNING
        
    async def initialize(self) -> None:
        """Initialize the component."""
        if self._state != ComponentState.INITIALIZING:
            raise ComponentError(f"Cannot initialize component in state {self._state}")
            
        self._logger.info(f"Initializing {self.name}...")
        
        try:
            await self._initialize()
            self._transition_to(ComponentState.INITIALIZED)
            self._logger.info(f"{self.name} initialized successfully")
        except Exception as e:
            self._transition_to(ComponentState.ERROR)
            self._logger.error(f"Failed to initialize {self.name}: {e}")
            raise ComponentError(f"Initialization failed: {e}") from e
            
    async def start(self) -> None:
        """Start the component."""
        if self._state != ComponentState.INITIALIZED:
            raise ComponentError(f"Cannot start component in state {self._state}")
            
        self._logger.info(f"Starting {self.name}...")
        self._transition_to(ComponentState.STARTING)
        
        try:
            await self._start()
            self._transition_to(ComponentState.RUNNING)
            self._logger.info(f"{self.name} started successfully")
        except Exception as e:
            self._transition_to(ComponentState.ERROR)
            self._logger.error(f"Failed to start {self.name}: {e}")
            raise ComponentError(f"Start failed: {e}") from e
            
    async def stop(self) -> None:
        """Stop the component."""
        if self._state not in (ComponentState.RUNNING, ComponentState.STARTING):
            self._logger.warning(f"Attempting to stop {self.name} in state {self._state}")
            return
            
        self._logger.info(f"Stopping {self.name}...")
        self._transition_to(ComponentState.STOPPING)
        
        try:
            await self._stop()
            self._transition_to(ComponentState.STOPPED)
            self._logger.info(f"{self.name} stopped successfully")
        except Exception as e:
            self._transition_to(ComponentState.ERROR)
            self._logger.error(f"Failed to stop {self.name}: {e}")
            raise ComponentError(f"Stop failed: {e}") from e
            
    async def resume(self) -> None:
        """Resume the component from stopped state."""
        if self._state != ComponentState.STOPPED:
            raise ComponentError(f"Cannot resume component in state {self._state}")
            
        self._logger.info(f"Resuming {self.name}...")
        self._transition_to(ComponentState.RESUMING)
        
        try:
            await self._resume()
            self._transition_to(ComponentState.RUNNING)
            self._logger.info(f"{self.name} resumed successfully")
        except Exception as e:
            self._transition_to(ComponentState.ERROR)
            self._logger.error(f"Failed to resume {self.name}: {e}")
            raise ComponentError(f"Resume failed: {e}") from e
            
    async def dispose(self) -> None:
        """Dispose of the component and cleanup resources."""
        if self._state == ComponentState.DISPOSED:
            return
            
        self._logger.info(f"Disposing {self.name}...")
        
        # Stop if running
        if self._state in (ComponentState.RUNNING, ComponentState.STARTING):
            await self.stop()
            
        try:
            await self._dispose()
            self._transition_to(ComponentState.DISPOSED)
            self._logger.info(f"{self.name} disposed successfully")
        except Exception as e:
            self._transition_to(ComponentState.ERROR)
            self._logger.error(f"Failed to dispose {self.name}: {e}")
            raise ComponentError(f"Dispose failed: {e}") from e
    
    def _transition_to(self, new_state: ComponentState) -> None:
        """Transition to a new state."""
        old_state = self._state
        self._state = new_state
        self._state_history.append(new_state)
        
        self._logger.debug(f"State transition: {old_state} -> {new_state}")
        
    # Abstract methods to be implemented by subclasses
    @abstractmethod
    async def _initialize(self) -> None:
        """Component-specific initialization logic."""
        pass
        
    @abstractmethod
    async def _start(self) -> None:
        """Component-specific start logic."""
        pass
        
    @abstractmethod
    async def _stop(self) -> None:
        """Component-specific stop logic."""
        pass
        
    async def _resume(self) -> None:
        """Component-specific resume logic. Default implementation calls start."""
        await self._start()
        
    async def _dispose(self) -> None:
        """Component-specific dispose logic. Override if cleanup needed."""
        pass
        
    def __repr__(self) -> str:
        return f"{self.__class__.__name__}(name='{self.name}', state={self._state})"
