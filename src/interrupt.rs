//! Interrupt handling with disciplined approach
//! Follows: Interrupt Handling Discipline, Direct Device Control

use std::sync::atomic::{AtomicBool, Ordering};

/// Interrupt handler registry
pub struct InterruptHandler {
    enabled: AtomicBool,
    handler_count: AtomicBool,
}

impl InterruptHandler {
    pub fn new() -> Self {
        InterruptHandler {
            enabled: AtomicBool::new(false),
            handler_count: AtomicBool::new(false),
        }
    }

    /// Enable interrupt handling
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    /// Disable interrupt handling
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    /// Check if interrupts are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Handle interrupt
    pub fn handle(&self, interrupt_num: u32) -> Result<(), &'static str> {
        if !self.is_enabled() {
            return Err("Interrupts disabled");
        }
        
        // Process interrupt
        log::debug!("Handling interrupt: {}", interrupt_num);
        Ok(())
    }
}

impl Default for InterruptHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_handling() {
        let handler = InterruptHandler::new();
        assert!(!handler.is_enabled());
        handler.enable();
        assert!(handler.is_enabled());
        let result = handler.handle(1);
        assert!(result.is_ok());
    }
}
