use kable::log_result;

pub struct ParameterTest;

impl ParameterTest {
    // Test default parameters
    #[log_result]
    pub fn default_config(&self) -> Result<String, String> {
        Ok("default".to_string())
    }

    // Test log_values = true
    #[log_result(log_values = true)]
    pub fn with_values(&self) -> Result<i32, String> {
        Ok(123)
    }

    // Test max_length parameter
    #[log_result(log_values = true, max_length = 20)]
    pub fn short_max_length(&self) -> Result<String, String> {
        Ok("This is a very long string that should definitely be truncated".to_string())
    }

    // Test debug_only = false (should log in release builds too)
    #[log_result(log_values = true, debug_only = false)]
    pub fn non_debug_only(&self) -> Result<bool, String> {
        Ok(true)
    }

    // Test all parameters together
    #[log_result(log_values = true, max_length = 100, debug_only = true)]
    pub fn all_parameters(&self) -> Result<Vec<u8>, String> {
        Ok(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
    }

    // Test with complex return types
    #[log_result(log_values = true, max_length = 200)]
    pub fn complex_return_type(&self) -> Result<std::collections::HashMap<String, Vec<i32>>, String> {
        let mut map = std::collections::HashMap::new();
        map.insert("numbers".to_string(), vec![1, 2, 3, 4, 5]);
        map.insert("more_numbers".to_string(), vec![6, 7, 8, 9, 10]);
        Ok(map)
    }

    // Test error case with parameters
    #[log_result(log_values = true)]
    pub fn error_with_params(&self) -> Result<String, String> {
        Err("This error should be logged with macro_debug prefix".to_string())
    }
}

#[tokio::test]
async fn test_parameter_defaults() {
    let test = ParameterTest;
    
    let result = test.default_config();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "default");
}

#[tokio::test]
async fn test_value_logging_parameter() {
    let test = ParameterTest;
    
    let result = test.with_values();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 123);
}

#[tokio::test]
async fn test_max_length_parameter() {
    let test = ParameterTest;
    
    let result = test.short_max_length();
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.len() > 20); // Original string is longer than max_length
}

#[tokio::test]
async fn test_debug_only_parameter() {
    let test = ParameterTest;
    
    let result = test.non_debug_only();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_all_parameters() {
    let test = ParameterTest;
    
    let result = test.all_parameters();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
}

#[tokio::test]
async fn test_complex_types() {
    let test = ParameterTest;
    
    let result = test.complex_return_type();
    assert!(result.is_ok());
    let map = result.unwrap();
    assert!(map.contains_key("numbers"));
    assert!(map.contains_key("more_numbers"));
}

#[tokio::test]
async fn test_error_logging() {
    let test = ParameterTest;
    
    let result = test.error_with_params();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "This error should be logged with macro_debug prefix");
}
