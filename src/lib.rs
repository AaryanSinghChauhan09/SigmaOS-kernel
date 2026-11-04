//! SigmaOS-kernel - Core Kernel Component for SigmaOS
#![allow(dead_code)]

pub mod config;
pub mod error;
pub mod service;

pub use config::Config;
pub use error::{ComponentError, Result};
pub use service::Service;

/// Core Kernel abstraction for SigmaOS.
/// Follows Minimal Kernel Philosophy and Clear Abstraction principles.
#[derive(Debug)]
pub struct Kernel {
    pub config: Config,
}

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
        let mut config = Config::default();
        config.enabled = false;
        let k = Kernel::new(config);
        assert!(!k.is_enabled());
    }
}
