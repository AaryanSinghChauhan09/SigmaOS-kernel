#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelState { Booting, Initialized, Halting, Panic }

#[derive(Debug)]
pub struct KernelInstance {
    pub state: KernelState,
    pub uptime_ticks: u64,
}

impl Default for KernelInstance {
    fn default() -> Self {
        Self { state: KernelState::Initialized, uptime_ticks: 0 }
    }
}

impl KernelInstance {
    pub fn new() -> Self { Self::default() }
    pub fn tick(&mut self) { self.uptime_ticks += 1; }
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
