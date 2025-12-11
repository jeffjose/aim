//! Tests for error types

#[cfg(test)]
mod tests {
    use crate::error::{AdbError, AimError};

    #[test]
    fn test_adb_error_display() {
        let err = AdbError::NoDevicesFound;
        assert!(format!("{}", err).contains("No devices"));

        let err = AdbError::DeviceNotFound("abc123".to_string());
        assert!(format!("{}", err).contains("abc123"));

        let err = AdbError::DeviceIdRequired;
        assert!(format!("{}", err).contains("device"));
    }

    #[test]
    fn test_aim_error_display() {
        let err = AimError::NoDevicesFound;
        assert!(format!("{}", err).contains("No devices"));

        let err = AimError::DeviceNotFound("xyz789".to_string());
        assert!(format!("{}", err).contains("xyz789"));

        let err = AimError::MultipleDevicesFound;
        assert!(format!("{}", err).contains("Multiple"));

        let err = AimError::DeviceIdRequired;
        assert!(format!("{}", err).contains("required"));
    }

    #[test]
    fn test_aim_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let aim_err: AimError = io_err.into();
        assert!(format!("{}", aim_err).contains("ADB connection error"));
    }

    #[test]
    fn test_aim_error_from_string() {
        let string_err = "something went wrong".to_string();
        let aim_err: AimError = string_err.into();
        assert!(format!("{}", aim_err).contains("something went wrong"));
    }
}
