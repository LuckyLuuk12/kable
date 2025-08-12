use kable::{log_result, log_errors_only};

pub struct TestService {
    pub name: String,
}

impl TestService {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    #[log_result]
    pub fn basic_method(&self) -> Result<String, String> {
        Ok(format!("Hello from {}", self.name))
    }

    #[log_errors_only]
    pub fn error_only_method(&self) -> Result<i32, String> {
        Err("Something went wrong".to_string())
    }

    #[log_result(log_values = true, max_length = 30)]
    pub fn custom_context_method(&self) -> Result<Vec<String>, String> {
        Ok(vec!["item1".to_string(), "item2".to_string(), "item3".to_string()])
    }

    #[log_result(log_values = true)]
    pub fn instance_method(&self) -> Result<bool, String> {
        Ok(true)
    }

    #[log_result(log_values = true)]
    pub async fn async_method(&self) -> Result<String, String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        Ok(format!("Async result from {}", self.name))
    }

    #[log_result]
    pub fn multi_param_method(&self, value: i32, flag: bool) -> Result<String, String> {
        if flag {
            Ok(format!("Value: {} from {}", value, self.name))
        } else {
            Err("Flag was false".to_string())
        }
    }
}

// Test trait implementations with macros
pub trait TestTrait {
    fn trait_method(&self) -> Result<String, String>;
}

impl TestTrait for TestService {
    #[log_result]
    fn trait_method(&self) -> Result<String, String> {
        Ok(format!("Trait method from {}", self.name))
    }
}

// Static methods
impl TestService {
    #[log_result]
    pub fn static_method() -> Result<String, String> {
        Ok("Static method result".to_string())
    }

    #[log_errors_only]
    pub fn static_error_method() -> Result<String, String> {
        Err("Static method error".to_string())
    }
}

#[tokio::test]
async fn test_impl_block_basic() {
    let service = TestService::new("test_service".to_string());
    
    let result = service.basic_method();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello from test_service");
    
    let result = service.error_only_method();
    assert!(result.is_err());
    
    let result = service.instance_method();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_impl_block_advanced() {
    let service = TestService::new("advanced_test".to_string());
    
    let result = service.custom_context_method();
    assert!(result.is_ok());
    
    let result = service.multi_param_method(42, true);
    assert!(result.is_ok());
    
    let result = service.multi_param_method(42, false);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_impl_methods() {
    let service = TestService::new("async_test".to_string());
    
    let result = service.async_method().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Async result from async_test");
}

#[tokio::test]
async fn test_trait_implementations() {
    let service = TestService::new("trait_test".to_string());
    
    let result = service.trait_method();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Trait method from trait_test");
}

#[tokio::test]
async fn test_static_methods() {
    let result = TestService::static_method();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Static method result");
    
    let result = TestService::static_error_method();
    assert!(result.is_err());
}
