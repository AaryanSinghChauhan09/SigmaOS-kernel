/// Kernel state following Minimal Kernel Philosophy
/// - Booting: System is booting
/// - Initialized: System is ready
/// - Halting: System is shutting down
/// - Panic: System has encountered a critical error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelState { Booting, Initialized, Halting, Panic }

/// Kernel instance following Single Responsibility principle
/// - Manages kernel state
/// - Tracks uptime
/// - Provides operational status
#[derive(Debug)]
pub struct KernelInstance {
    pub state: KernelState,
    pub uptime_ticks: u64,
}

impl Default for KernelInstance {
    // Follows Secure Defaults: system starts in Initialized state
    fn default() -> Self {
        Self { state: KernelState::Initialized, uptime_ticks: 0 }
    }
}

impl KernelInstance {
    /// Creates a new kernel instance
    /// Follows KISS principle: simple, straightforward initialization
    pub fn new() -> Self { Self::default() }
    
    /// Increments uptime counter
    /// Follows Fail-Fast: simple, direct operation
    pub fn tick(&mut self) { self.uptime_ticks += 1; }
    
    /// Checks if kernel is operational
    /// Follows KISS principle: simple state check
    pub fn is_operational(&self) -> bool { self.state == KernelState::Initialized }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_kernel_instance_lifecycle() {
        let mut k = KernelInstance::new();
        assert!(k.is_operational());
        k.tick();
        assert_eq!(k.uptime_ticks, 1);
    }
}
