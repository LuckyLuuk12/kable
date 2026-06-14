use proc_macro::TokenStream;

mod caching;
mod logging;

// Caching proc-macros:
#[proc_macro_attribute]
pub fn persistent_cache(attr: TokenStream, item: TokenStream) -> TokenStream {
    caching::persistent_cache(attr, item)
}

// Logging proc-macros:
#[proc_macro_attribute]
pub fn log_result(attr: TokenStream, item: TokenStream) -> TokenStream {
    logging::log_result(attr, item)
}
#[proc_macro_attribute]
pub fn log_errors_only(attr: TokenStream, item: TokenStream) -> TokenStream {
    logging::log_errors_only(attr, item)
}
#[proc_macro_attribute]
pub fn log_result_custom(attr: TokenStream, item: TokenStream) -> TokenStream {
    logging::log_result_custom(attr, item)
}
#[proc_macro_attribute]
pub fn log_result_with_instance(attr: TokenStream, item: TokenStream) -> TokenStream {
    logging::log_result_with_instance(attr, item)
}
#[proc_macro_attribute]
pub fn log_errors_only_with_instance(attr: TokenStream, item: TokenStream) -> TokenStream {
    logging::log_errors_only_with_instance(attr, item)
}
