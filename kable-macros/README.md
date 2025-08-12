# Kable Logging Macros

This crate provides procedural macros for automatic error logging in Rust functions that return `Result<T, E>`. The macros integrate seamlessly with the Kable logging system.

## Macros

### `#[log_result]`
Logs both successful completions and errors using the global logging system.

```rust
use kable_macros::log_result;

#[log_result]
fn my_function() -> Result<String, String> {
    if some_condition {
        Ok("Success!".to_string())
    } else {
        Err("Something went wrong".to_string())
    }
}
```

**Output (to both console and frontend):**
- Success: `Function 'my_function' completed successfully`
- Error: `Function 'my_function' failed: Something went wrong`

### `#[log_errors_only]`
Only logs errors, no success messages.

```rust
use kable_macros::log_errors_only;

#[log_errors_only]
fn my_function() -> Result<i32, String> {
    // Only failures will be logged
    Ok(42) // No log output
    // Err("error") // Would log: Function 'my_function' failed: error
}
```

### `#[log_result_custom("context")]`
Logs with custom context messages.

```rust
use kable_macros::log_result_custom;

#[log_result_custom("Database operation")]
fn connect_db() -> Result<Connection, String> {
    // Custom context in logs
}
```

**Output:**
- Success: `Database operation completed successfully in function 'connect_db'`
- Error: `Database operation in function 'connect_db': Connection failed`

### `#[log_result_with_instance("instance_id")]`
Logs both success and errors with an instance_id for installation-specific tracking.

```rust
use kable_macros::log_result_with_instance;

#[log_result_with_instance("forge-1.20.1")]
fn install_mod() -> Result<String, String> {
    // Installation-specific logging
}
```

**Output:**
- Success: `Function 'install_mod' completed successfully` (with instance_id: "forge-1.20.1")
- Error: `Function 'install_mod' failed: Download timeout` (with instance_id: "forge-1.20.1")

### `#[log_errors_only_with_instance("instance_id")]`
Only logs errors with an instance_id for installation-specific tracking.

```rust
use kable_macros::log_errors_only_with_instance;

#[log_errors_only_with_instance("vanilla-1.21")]
fn load_mods() -> Result<Vec<String>, String> {
    // Only errors logged with instance tracking
}
```

## Integration with Kable Logging System

The macros automatically use the Kable logging system:
- **Frontend Integration**: Logs are sent to the frontend via Tauri events
- **Persistent Storage**: Logs are written to files when persistent logging is enabled
- **Instance Tracking**: Installation-specific logs can be tracked separately
- **Log Levels**: Errors use `error_global()`, successes use `debug_global()`

## How It Works

The macros automatically wrap your function with logging code:

1. Create an internal version of your function
2. Call the internal function
3. Check the result and log accordingly using `crate::logging::Logger`
4. Return the original result unchanged

The macros only activate for functions that return `Result<T, E>`. Other functions are left unchanged.

## Usage in Kable Project

In the main Kable project, import and use the macros like this:

```rust
use kable_macros::{log_result, log_errors_only, log_result_custom, log_result_with_instance};

#[log_result]
pub async fn download_mod(url: &str) -> Result<Vec<u8>, String> {
    // Automatic logging to frontend and persistent storage
}

#[log_errors_only]
fn parse_config() -> Result<Config, ConfigError> {
    // Only errors logged
}

#[log_result_custom("Authentication")]
pub async fn authenticate_user(token: &str) -> Result<User, AuthError> {
    // Custom context in logs
}

#[log_result_with_instance("forge-1.20.1")]
pub async fn install_forge_mod(mod_id: &str) -> Result<(), String> {
    // Installation-specific logging
}
```

The macros will automatically add appropriate logging to these functions without requiring you to manually add logging code to each function. All logs will be properly integrated with the Kable logging system, appearing in both the console and the frontend UI.
