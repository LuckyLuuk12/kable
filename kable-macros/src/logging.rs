use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType, Type, TypePath};

use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Result, Token,
};

#[derive(Default)]
pub struct LogConfig {
    pub success: bool,
    pub values: bool,
    pub instance: bool,
    pub context: Option<String>,
}

impl Parse for LogConfig {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut cfg = Self::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            match ident.to_string().as_str() {
                "success" => {
                    cfg.success = true;
                }

                "values" => {
                    cfg.values = true;
                }

                "instance" => {
                    cfg.instance = true;
                }

                "context" => {
                    input.parse::<Token![=]>()?;
                    let value: LitStr = input.parse()?;
                    cfg.context = Some(value.value());
                }

                _ => {
                    return Err(syn::Error::new(ident.span(), "unknown log option"));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(cfg)
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let cfg = if attr.is_empty() { LogConfig::default() } else { parse_macro_input!(attr as LogConfig) };

    let input = parse_macro_input!(item as ItemFn);

    if !returns_result(&input) {
        return quote!(#input).into();
    }

    build_wrapper(input, cfg)
}

fn build_wrapper(func: ItemFn, cfg: LogConfig) -> TokenStream {
    let vis = &func.vis;
    let sig = &func.sig;
    let attrs = &func.attrs;
    let block = &func.block;

    let fn_name = sig.ident.to_string();

    let instance_expr = if cfg.instance { first_arg_expr(&func) } else { quote!(None) };

    let context = cfg.context.unwrap_or_default();

    let success_branch = if cfg.success {
        if cfg.values {
            quote! {
                crate::logging::log_success(
                    #fn_name,
                    Some(format!("{:?}", value)),
                    #instance_expr,
                    #context,
                );
            }
        } else {
            quote! {
                crate::logging::log_success(
                    #fn_name,
                    None,
                    #instance_expr,
                    #context,
                );
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let result = (async move #block).await;

            match &result {
                Ok(value) => {
                    #success_branch
                }
                Err(error) => {
                    crate::logging::log_error(
                        #fn_name,
                        error,
                        #instance_expr,
                        #context,
                    );
                }
            }

            result
        }
    };

    expanded.into()
}

fn first_arg_expr(func: &ItemFn) -> proc_macro2::TokenStream {
    let Some(first) = func.sig.inputs.first() else {
        return quote!(None);
    };

    match first {
        FnArg::Typed(arg) => {
            if let Pat::Ident(ident) = arg.pat.as_ref() {
                let name = &ident.ident;

                quote! {
                    Some(#name.to_string())
                }
            } else {
                quote!(None)
            }
        }

        _ => quote!(None),
    }
}

fn returns_result(func: &ItemFn) -> bool {
    match &func.sig.output {
        ReturnType::Type(_, ty) => is_result(ty),
        ReturnType::Default => false,
    }
}

fn is_result(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => path.segments.last().map(|s| s.ident == "Result").unwrap_or(false),
        _ => false,
    }
}
