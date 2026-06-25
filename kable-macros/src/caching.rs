use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Ident, ItemFn, Lit, Token};

/// --------------------------------
/// Cache macro configuration
/// --------------------------------
#[derive(Debug, Clone)]
struct CacheConfig {
    parent: String,
    ttl_secs: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self { parent: "default".to_string(), ttl_secs: Some(86400) }
    }
}

/// --------------------------------
/// Attribute parser (STRICT)
/// --------------------------------
impl Parse for CacheConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = CacheConfig::default();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "parent" => {
                    let Lit::Str(s) = input.parse()? else {
                        return Err(syn::Error::new_spanned(key, "expected string literal for `parent`"));
                    };
                    config.parent = s.value();
                }

                "ttl_secs" => {
                    let Lit::Int(i) = input.parse()? else {
                        return Err(syn::Error::new_spanned(key, "expected integer literal for `ttl_secs`"));
                    };

                    config.ttl_secs = Some(i.base10_parse()?);
                }

                other => {
                    return Err(syn::Error::new_spanned(key, format!("unknown cache attribute: `{other}`")));
                }
            }

            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>();
            }
        }

        Ok(config)
    }
}

/// --------------------------------
/// Persistent file-system cache for function results.
///
/// This macro wraps a function and caches its result on disk using a deterministic key
/// derived from the function name + arguments.
///
/// ## Behavior
///
/// - On first call: executes function and stores result in:
///   `.kable/cache/<parent>/<hash>.bin`
/// - On subsequent calls:
///   - returns cached value if present
///   - TTL not expired
///   - entry format valid
/// - Otherwise recomputes and overwrites cache entry
///
/// ## Cache structure
///
/// <pre>
/// .kable/
///   cache/
///     &lt;parent&gt;/
///       &lt;hash(args)&gt;.bin
/// </pre>
///
/// ## Attributes
///
/// - `parent = "name"`: cache namespace/group
/// - `ttl_secs = 60`: optional time-to-live in seconds
///
/// ## Example
///
/// ```rust
/// #[persistent_cache(parent = "versions", ttl_secs = 300)]
/// async fn get_versions() -> Result<Vec<String>, Error> {
///     fetch_versions_from_network().await
/// }
/// ```
///
/// ## Notes
///
/// - Function arguments must implement `serde::Serialize`
/// - Cache is async-safe and uses per-key locking
/// - Requires runtime: `kable_cache::__macro_get_or_compute`
pub fn persistent_cache(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let config: CacheConfig = if attr.is_empty() { CacheConfig::default() } else { parse_macro_input!(attr as CacheConfig) };

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let attrs = &input_fn.attrs;
    let block = &input_fn.block;

    let fn_name = sig.ident.to_string();
    let parent = config.parent;
    let ttl = config.ttl_secs;
    let ttl_expr = match ttl {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    };

    // Extract argument patterns + identifiers correctly
    let inputs = &sig.inputs;

    let arg_names: Vec<Ident> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    return Some(pat_ident.ident.clone());
                }
            }
            None
        })
        .collect();

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let __cache_args = (
                #(#arg_names.clone(),)*
            );

            crate::system::cache::__macro_get_or_compute(
                #parent,
                ( #fn_name, __cache_args ),
                #ttl_expr,
                || async move #block
            ).await
        }
    };

    TokenStream::from(expanded)
}
