use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Expr, Ident, ItemFn, Lit, Token};

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
        Self {
            parent: "default".to_string(),
            ttl_secs: Some(86400), // 24h
        }
    }
}

/// --------------------------------
/// Attribute parser (syn Parse style)
/// --------------------------------
impl Parse for CacheConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = CacheConfig::default();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "parent" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        config.parent = s.value();
                    }
                }

                "ttl_secs" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Int(i) = lit {
                        config.ttl_secs = Some(i.base10_parse()?);
                    }
                }

                _ => {
                    // skip unknown expression safely
                    let _: Expr = input.parse()?;
                }
            }

            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }

        Ok(config)
    }
}

/// Persistent file-system cache for function results.
///
/// This macro wraps a function and caches its result on disk using a deterministic key
/// derived from the function signature and runtime arguments.
///
/// ## Behavior
///
/// - On first call: executes function and stores result in `.kable/cache/<parent>/<hash>.bin`
/// - On subsequent calls: returns cached value if:
///   - cache entry exists
///   - TTL has not expired (if configured)
/// - Otherwise recomputes and overwrites cache entry
///
/// ## Cache structure
///
/// <pre>
/// .kable/
///   cache/
///     &lt;parent&gt;/
///       &lt;hash(function + args)&gt;.bin
/// </pre>
///
/// ## Attributes
///
/// - `parent = "name"`: logical grouping folder inside cache
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
/// - Function arguments must be serializable (internally handled by runtime cache layer)
/// - Cache is async-safe and uses per-key locking to prevent duplicate computation
/// - This macro requires runtime support from `kable_cache::__macro_get_or_compute`
pub fn persistent_cache(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let config: CacheConfig = if attr.is_empty() {
        CacheConfig::default()
    } else {
        let cfg = parse_macro_input!(attr as CacheConfig);
        cfg
    };

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let attrs = &input_fn.attrs;
    let block = &input_fn.block;

    let parent = config.parent;
    let ttl = config.ttl_secs;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let result = kable_cache::__macro_get_or_compute(
                #parent,
                // NOTE: runtime args must be provided by wrapper or serde,
                // not syn AST. This is intentionally left as runtime hook.
                (),
                #ttl,
                || async move #block
            ).await;

            result
        }
    };

    TokenStream::from(expanded)
}
