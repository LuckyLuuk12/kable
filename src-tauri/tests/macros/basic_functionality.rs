use kable::log_result;

pub struct SimpleTest;

impl SimpleTest {
    #[log_result]
    pub fn test_method(&self) -> Result<String, String> {
        Ok("Success".to_string())
    }

    #[log_result]
    pub fn error_method(&self) -> Result<String, String> {
        Err("This is an error".to_string())
    }

    // Test with return value logging enabled
    #[log_result(log_values = true)]
    pub fn value_logging_method(&self) -> Result<i32, String> {
        Ok(42)
    }

    // Test with custom max length
    #[log_result(log_values = true, max_length = 50)]
    pub fn long_value_method(&self) -> Result<String, String> {
        Ok("This is a very long string that should be truncated when logged because it exceeds the max length".to_string())
    }

    // Test with both parameters
    #[log_result(log_values = true, debug_only = false, max_length = 100)]
    pub fn full_config_method(&self) -> Result<Vec<i32>, String> {
        Ok(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    }

    #[log_result] 
    pub async fn async_test(&self) -> Result<String, String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        Ok("Async success".to_string())
    }
}

#[tokio::test]
async fn test_basic_macros() {
    let test = SimpleTest;
    
    // Test successful method
    let result = test.test_method();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");
    
    // Test error method
    let result = test.error_method();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "This is an error");
}

#[tokio::test]
async fn test_return_value_logging() {
    let test = SimpleTest;
    
    // Test value logging
    let result = test.value_logging_method();
    assert_eq!(result.unwrap(), 42);
    
    // Test long value truncation
    let result = test.long_value_method();
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.len() > 50); // Original string is longer than 50
    
    // Test full configuration
    let result = test.full_config_method();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

#[tokio::test]
async fn test_async_macros() {
    let test = SimpleTest;
    
    // Test async method
    let result = test.async_test().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Async success");
}
