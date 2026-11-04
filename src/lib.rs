use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Config {
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Core Kernel abstraction for SigmaOS.
/// Follows Minimal Kernel Philosophy and Clear Abstraction principles.
#[derive(Debug)]
pub struct Kernel {
    pub config: Config,
}

pub type kernel = Kernel;

impl Kernel {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_default_initialization() {
        let config = Config::default();
        let mut k = Kernel::new(config);
        assert!(k.is_enabled());
        assert!(k.initialize().is_ok());
    }

    #[test]
    fn test_kernel_custom_config() {
        let config = Config { enabled: false };
        let k = Kernel::new(config);
        assert!(!k.is_enabled());
    }
}
