//! Kernel lifecycle state.

/// High-level kernel state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelState {
    /// Early boot, before userspace is ready.
    Booting,
    /// Fully initialized and operational.
    Initialized,
    /// Shutting down.
    Halting,
    /// Unrecoverable error.
    Panic,
}

/// Kernel instance: state + uptime + panic info.
#[derive(Debug)]
pub struct KernelInstance {
    pub state: KernelState,
    pub uptime_ticks: u64,
    pub panic_message: Option<String>,
}

impl Default for KernelInstance {
    fn default() -> Self {
        Self { state: KernelState::Initialized, uptime_ticks: 0, panic_message: None }
    }
}

impl KernelInstance {
    pub fn new() -> Self { Self::default() }
    pub fn tick(&mut self) { self.uptime_ticks += 1; }
    pub fn is_operational(&self) -> bool { self.state == KernelState::Initialized }
    /// Transition to a new state; panics are sticky.
    pub fn set_state(&mut self, s: KernelState) {
        if self.state != KernelState::Panic { self.state = s; }
    }
    /// Record a panic; kernel enters Panic state.
    pub fn panic(&mut self, msg: impl Into<String>) {
        self.panic_message = Some(msg.into());
        self.state = KernelState::Panic;
    }
}
