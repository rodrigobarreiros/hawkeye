use crate::domain::errors::{DomainError, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct ServerConfig {
    port: u16,
    mount_point: String,
    enable_looping: bool,
}

impl ServerConfig {
    pub fn new(port: u16, mount_point: String) -> Result<Self> {
        Self::validate_port(port)?;
        Self::validate_mount_point(&mount_point)?;

        Ok(Self {
            port,
            mount_point,
            enable_looping: true,
        })
    }

    pub fn with_looping(mut self, enabled: bool) -> Self {
        self.enable_looping = enabled;
        self
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn mount_point(&self) -> &str {
        &self.mount_point
    }

    pub fn looping_enabled(&self) -> bool {
        self.enable_looping
    }

    fn validate_port(port: u16) -> Result<()> {
        if port == 0 {
            return Err(DomainError::InvalidPort);
        }
        Ok(())
    }

    fn validate_mount_point(mount_point: &str) -> Result<()> {
        if !mount_point.starts_with('/') {
            return Err(DomainError::InvalidMountPoint(mount_point.to_string()));
        }
        if mount_point.len() > 100 {
            return Err(DomainError::MountPointTooLong);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_zero_port() {
        let result = ServerConfig::new(0, "/cam1".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::InvalidPort));
    }

    #[test]
    fn test_rejects_invalid_mount_point() {
        let result = ServerConfig::new(8554, "cam1".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::InvalidMountPoint(_)
        ));
    }

    #[test]
    fn test_rejects_long_mount_point() {
        let long_mount = format!("/{}", "a".repeat(100));
        let result = ServerConfig::new(8554, long_mount);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::MountPointTooLong));
    }

    #[test]
    fn test_accepts_valid_config() {
        let result = ServerConfig::new(8554, "/cam1".to_string());
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.port(), 8554);
        assert_eq!(config.mount_point(), "/cam1");
        assert!(config.looping_enabled());
    }

    #[test]
    fn test_with_looping_disabled() {
        let config = ServerConfig::new(8554, "/cam1".to_string())
            .unwrap()
            .with_looping(false);

        assert!(!config.looping_enabled());
    }
}
