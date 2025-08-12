use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse::Parse, parse::ParseStream, ItemFn, ReturnType, Type, TypePath, Lit, Meta, Expr, Token, Ident};

// Configuration for macro behavior
#[derive(Debug, Clone)]
struct LogConfig {
    log_values: bool,
    max_length: usize,
    debug_only: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_values: false,
            max_length: 200,
            debug_only: true,
        }
    }
}

// Custom parser for macro attributes
struct MacroArgs {
    config: LogConfig,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = LogConfig::default();
        
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            
            match ident.to_string().as_str() {
                "log_values" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Bool(lit_bool) = lit {
                        config.log_values = lit_bool.value;
                    }
                }
                "max_length" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Int(lit_int) = lit {
                        if let Ok(val) = lit_int.base10_parse::<usize>() {
                            config.max_length = val;
                        }
                    }
                }
                "debug_only" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Bool(lit_bool) = lit {
                        config.debug_only = lit_bool.value;
                    }
                }
                _ => {
                    // Skip unknown parameters
                    let _: Expr = input.parse()?;
                }
            }
            
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        
        Ok(MacroArgs { config })
    }
}

/// A simple macro that adds a wrapper around your function that logs errors.
/// For functions that return Result<T, E>, it will log errors automatically.
/// 
/// Parameters:
/// - log_values: bool (default: false) - Log return values for successful calls
/// - max_length: usize (default: 200) - Maximum length for logged values
/// - debug_only: bool (default: true) - Only log values in debug builds
/// 
/// Examples:
/// - #[log_result] - Basic error logging
/// - #[log_result(log_values = true)] - Log errors and return values
/// - #[log_result(log_values = true, max_length = 500)] - Custom max length
#[proc_macro_attribute]
pub fn log_result(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let config = if attr.is_empty() {
        LogConfig::default()
    } else {
        match syn::parse::<MacroArgs>(attr) {
            Ok(args) => args.config,
            Err(_) => LogConfig::default(),
        }
    };
    
    generate_logging_wrapper_with_config(&input_fn, config, true)
}

/// Only logs errors, not success
/// Parameters: Same as log_result but only applies to error cases
#[proc_macro_attribute]
pub fn log_errors_only(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let config = if attr.is_empty() {
        LogConfig::default()
    } else {
        match syn::parse::<MacroArgs>(attr) {
            Ok(args) => args.config,
            Err(_) => LogConfig::default(),
        }
    };
    
    generate_logging_wrapper_with_config(&input_fn, config, false)
}

/// Logs with custom context message
#[proc_macro_attribute]
pub fn log_result_custom(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let context = attr.to_string().trim_matches('"').to_string();
    
    generate_custom_logging_wrapper(&input_fn, context)
}

/// Logs with instance_id for installation-specific logging
#[proc_macro_attribute]
pub fn log_result_with_instance(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let instance_id = attr.to_string().trim_matches('"').to_string();
    
    generate_instance_logging_wrapper(&input_fn, instance_id, true)
}

/// Logs errors only with instance_id for installation-specific logging
#[proc_macro_attribute]
pub fn log_errors_only_with_instance(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let instance_id = attr.to_string().trim_matches('"').to_string();
    
    generate_instance_logging_wrapper(&input_fn, instance_id, false)
}

fn generate_logging_wrapper_with_config(input_fn: &ItemFn, config: LogConfig, log_success: bool) -> TokenStream {
    // Check if the function returns a Result type
    let returns_result = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => is_result_type(ty),
        _ => false,
    };
    
    if !returns_result {
        // If it's not a Result type, return the function unchanged
        return quote! { #input_fn }.into();
    }
    
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let attrs = &input_fn.attrs;
    let block = &input_fn.block;
    let is_async = input_fn.sig.asyncness.is_some();
    let max_length = config.max_length;
    
    let logging_code = if log_success {
        if config.log_values {
            if config.debug_only {
                quote! {
                    #[cfg(debug_assertions)]
                    {
                        match &result {
                            Err(e) => {
                                crate::logging::Logger::error_global(
                                    &format!("macro_debug: Function '{}' failed: {}", #fn_name_str, e),
                                    None
                                );
                            },
                            Ok(val) => {
                                let val_str = format!("{:?}", val);
                                let truncated = if val_str.len() > #max_length {
                                    format!("{}...", &val_str[..#max_length])
                                } else {
                                    val_str
                                };
                                crate::logging::Logger::debug_global(
                                    &format!("macro_debug: Function '{}' completed successfully, returned: {}", #fn_name_str, truncated),
                                    None
                                );
                            }
                        }
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        match &result {
                            Err(e) => {
                                crate::logging::Logger::error_global(
                                    &format!("macro_debug: Function '{}' failed: {}", #fn_name_str, e),
                                    None
                                );
                            },
                            Ok(_) => {
                                crate::logging::Logger::debug_global(
                                    &format!("macro_debug: Function '{}' completed successfully", #fn_name_str),
                                    None
                                );
                            }
                        }
                    }
                }
            } else {
                quote! {
                    match &result {
                        Err(e) => {
                            crate::logging::Logger::error_global(
                                &format!("macro_debug: Function '{}' failed: {}", #fn_name_str, e),
                                None
                            );
                        },
                        Ok(val) => {
                            let val_str = format!("{:?}", val);
                            let truncated = if val_str.len() > #max_length {
                                format!("{}...", &val_str[..#max_length])
                            } else {
                                val_str
                            };
                            crate::logging::Logger::debug_global(
                                &format!("macro_debug: Function '{}' completed successfully, returned: {}", #fn_name_str, truncated),
                                None
                            );
                        }
                    }
                }
            }
        } else {
            quote! {
                match &result {
                    Err(e) => {
                        crate::logging::Logger::error_global(
                            &format!("macro_debug: Function '{}' failed: {}", #fn_name_str, e),
                            None
                        );
                    },
                    Ok(_) => {
                        crate::logging::Logger::debug_global(
                            &format!("macro_debug: Function '{}' completed successfully", #fn_name_str),
                            None
                        );
                    }
                }
            }
        }
    } else {
        quote! {
            if let Err(e) = &result {
                crate::logging::Logger::error_global(
                    &format!("macro_debug: Function '{}' failed: {}", #fn_name_str, e),
                    None
                );
            }
        }
    };
    
    let execution_block = if is_async {
        quote! {
            let result = async move #block.await;
        }
    } else {
        quote! {
            let result = (|| #block)();
        }
    };
    
    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            #execution_block
            #logging_code
            result
        }
    };
    
    TokenStream::from(expanded)
}

fn generate_logging_wrapper(input_fn: &ItemFn, _logger_path: proc_macro2::TokenStream, log_success: bool) -> TokenStream {
    // Check if the function returns a Result type
    let returns_result = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => is_result_type(ty),
        _ => false,
    };
    
    if !returns_result {
        // If it's not a Result type, return the function unchanged
        return quote! { #input_fn }.into();
    }
    
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let attrs = &input_fn.attrs;
    let block = &input_fn.block;
    let is_async = input_fn.sig.asyncness.is_some();
    
    let logging_code = if log_success {
        quote! {
            match &result {
                Err(e) => {
                    crate::logging::Logger::error_global(
                        &format!("macro_debug: Function '{}' failed: {}", #fn_name_str, e),
                        None
                    );
                },
                Ok(_) => {
                    crate::logging::Logger::debug_global(
                        &format!("macro_debug: Function '{}' completed successfully", #fn_name_str),
                        None
                    );
                }
            }
        }
    } else {
        quote! {
            if let Err(e) = &result {
                crate::logging::Logger::error_global(
                    &format!("macro_debug: Function '{}' failed: {}", #fn_name_str, e),
                    None
                );
            }
        }
    };
    
    let execution_block = if is_async {
        quote! {
            let result = async move #block.await;
        }
    } else {
        quote! {
            let result = (|| #block)();
        }
    };
    
    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            #execution_block
            #logging_code
            result
        }
    };
    
    TokenStream::from(expanded)
}

fn generate_custom_logging_wrapper(input_fn: &ItemFn, context: String) -> TokenStream {
    // Check if the function returns a Result type
    let returns_result = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => is_result_type(ty),
        _ => false,
    };
    
    if !returns_result {
        return quote! { #input_fn }.into();
    }
    
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let attrs = &input_fn.attrs;
    let block = &input_fn.block;
    let is_async = input_fn.sig.asyncness.is_some();
    
    let execution_block = if is_async {
        quote! {
            let result = async move #block.await;
        }
    } else {
        quote! {
            let result = (|| #block)();
        }
    };
    
    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            #execution_block
            match &result {
                Err(e) => {
                    crate::logging::Logger::error_global(
                        &format!("macro_debug: {} in function '{}': {}", #context, #fn_name_str, e),
                        None
                    );
                },
                Ok(_) => {
                    crate::logging::Logger::debug_global(
                        &format!("macro_debug: {} completed successfully in function '{}'", #context, #fn_name_str),
                        None
                    );
                }
            }
            result
        }
    };
    
    TokenStream::from(expanded)
}

/// Helper function to check if a type is a Result type
fn is_result_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            path.segments.iter().any(|segment| {
                segment.ident == "Result"
            })
        }
        _ => false,
    }
}

fn generate_instance_logging_wrapper(input_fn: &ItemFn, instance_id: String, log_success: bool) -> TokenStream {
    // Check if the function returns a Result type
    let returns_result = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => is_result_type(ty),
        _ => false,
    };
    
    if !returns_result {
        return quote! { #input_fn }.into();
    }
    
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let attrs = &input_fn.attrs;
    let block = &input_fn.block;
    let is_async = input_fn.sig.asyncness.is_some();
    
    let logging_code = if log_success {
        quote! {
            match &result {
                Err(e) => {
                    crate::logging::Logger::error_global(
                        &format!("macro_debug: Function '{}' failed: {}", #fn_name_str, e),
                        Some(#instance_id)
                    );
                },
                Ok(_) => {
                    crate::logging::Logger::debug_global(
                        &format!("macro_debug: Function '{}' completed successfully", #fn_name_str),
                        Some(#instance_id)
                    );
                }
            }
        }
    } else {
        quote! {
            if let Err(e) = &result {
                crate::logging::Logger::error_global(
                    &format!("macro_debug: Function '{}' failed: {}", #fn_name_str, e),
                    Some(#instance_id)
                );
            }
        }
    };
    
    let execution_block = if is_async {
        quote! {
            let result = async move #block.await;
        }
    } else {
        quote! {
            let result = (|| #block)();
        }
    };
    
    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            #execution_block
            #logging_code
            result
        }
    };
    
    TokenStream::from(expanded)
}
