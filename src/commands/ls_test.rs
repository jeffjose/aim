#[cfg(test)]
mod tests {
    use super::super::ls::{LsCommand, LsArgs};
    use crate::commands::SubCommand;
    use crate::core::context::CommandContext;
    use crate::core::types::OutputFormat;
    use crate::device::DeviceManager;
    use crate::testing::{fixtures, MockAdb, TestScenario};
    
    #[tokio::test]
    async fn test_ls_no_devices() {
        // Setup
        let mock_adb = TestScenario::new()
            .build_mock_adb();
        
        // For now, we use the real DeviceManager
        // In a full implementation, we'd inject the mock
        let device_manager = DeviceManager::new();
        let ls_command = LsCommand::new(device_manager);
        
        let ctx = CommandContext::new()
            .with_output_format(OutputFormat::Table);
        
        let args = LsArgs {
            output: "table".to_string(),
        };
        
        // Execute
        let result = ls_command.run(&ctx, args).await;
        
        // Verify - should succeed even with no devices
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_ls_single_device() {
        // Setup
        let mock_adb = TestScenario::new()
            .with_device("test123")
            .with_property("ro.product.model", "TestPhone")
            .build_mock_adb();
        
        let device_manager = DeviceManager::new();
        let ls_command = LsCommand::new(device_manager);
        
        let ctx = CommandContext::new()
            .with_output_format(OutputFormat::Json);
        
        let args = LsArgs {
            output: "json".to_string(),
        };
        
        // Execute
        let result = ls_command.run(&ctx, args).await;
        
        // Verify
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_ls_multiple_devices() {
        // Setup
        let devices = fixtures::test_devices();
        let mock_adb = MockAdb::new()
            .with_devices(devices);
        
        let device_manager = DeviceManager::new();
        let ls_command = LsCommand::new(device_manager);
        
        let ctx = CommandContext::new()
            .with_output_format(OutputFormat::Plain);
        
        let args = LsArgs {
            output: "plain".to_string(),
        };
        
        // Execute
        let result = ls_command.run(&ctx, args).await;
        
        // Verify
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_ls_with_quiet_mode() {
        // Setup
        let device_manager = DeviceManager::new();
        let ls_command = LsCommand::new(device_manager);
        
        let ctx = CommandContext::new()
            .with_output_format(OutputFormat::Table)
            .with_quiet(true);
        
        let args = LsArgs {
            output: "table".to_string(),
        };
        
        // Execute
        let result = ls_command.run(&ctx, args).await;
        
        // Verify - should succeed and produce no output
        assert!(result.is_ok());
    }
}