use anyhow::Result;
use sigmaos_kernel_component::{Config, Kernel, Service};

#[test]
fn test_kernel_lifecycle_aaa() -> Result<()> {
    // Arrange
    let config = Config::default();
    let mut kernel = Kernel::new(config);

    // Act
    kernel.initialize()?;

    // Assert
    assert!(kernel.is_enabled());
    Ok(())
}

#[test]
fn test_service_lifecycle_aaa() -> Result<()> {
    // Arrange
    let config = Config::default();
    let mut service = Service::new(config);
    assert!(!service.is_running());

    // Act
    service.start()?;

    // Assert
    assert!(service.is_running());
    assert!(service.start().is_err()); // Double start fails

    // Act
    service.stop()?;

    // Assert
    assert!(!service.is_running());
    assert!(service.stop().is_err()); // Double stop fails
    Ok(())
}

#[test]
fn test_config_process_aaa() -> Result<()> {
    // Arrange
    let config = Config::new("debug");

    // Act
    let res = config.process();

    // Assert
    assert!(res.is_ok());
    Ok(())
}
