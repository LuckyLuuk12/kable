use std::env;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use walkdir::WalkDir;

struct TypeVisitor {
    types: Vec<String>,
    current_module: Vec<String>,
}

impl<'ast> Visit<'ast> for TypeVisitor {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        if is_facet_type(&i.attrs) {
            let name = &i.ident;
            let path = format!("{}::{}", self.current_module.join("::"), name);
            self.types.push(path);
        }
        // Continue visiting to find nested items if any
        syn::visit::visit_item_struct(self, i);
    }

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        if is_facet_type(&i.attrs) {
            let name = &i.ident;
            let path = format!("{}::{}", self.current_module.join("::"), name);
            self.types.push(path);
        }
        syn::visit::visit_item_enum(self, i);
    }

    // Handle inline modules possibly?
    // For now, assuming file-structure is the main module structure.
}

fn is_facet_type(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("derive") {
            let nested = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            );
            if let Ok(nested) = nested {
                for meta in nested {
                    if let syn::Meta::Path(path) = meta {
                        // Check for 'Facet' or 'facet::Facet'
                        // Simple string check on path segments
                        let segments: Vec<_> =
                            path.segments.iter().map(|s| s.ident.to_string()).collect();
                        if segments.contains(&"Facet".to_string()) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn main() {
    println!("cargo:rerun-if-changed=../api-types/src");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("register_types.rs");

    let api_types_src = Path::new("../api-types/src");
    let mut all_types = Vec::new();

    for entry in WalkDir::new(api_types_src) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "rs") {
            // Calculate module path
            // path relative to src: e.g. "auth.rs" -> "api_types::auth"
            // "lib.rs" -> "api_types"

            let relative = path.strip_prefix(api_types_src).unwrap();
            let mut modules = vec!["api_types".to_string()];

            let components: Vec<_> = relative
                .components()
                .map(|c| c.as_os_str().to_str().unwrap())
                .collect();

            // Handle lib.rs
            if components.len() == 1 && components[0] == "lib.rs" {
                // root module
            } else {
                for (i, component) in components.iter().enumerate() {
                    if i == components.len() - 1 {
                        // filename
                        let file_stem = Path::new(component).file_stem().unwrap().to_str().unwrap();
                        if file_stem != "mod" {
                            modules.push(file_stem.to_string());
                        }
                    } else {
                        // directory
                        modules.push(component.to_string());
                    }
                }
            }

            let content = fs::read_to_string(path).expect("Unable to read file");
            let file_ast = syn::parse_file(&content).expect("Unable to parse file");

            let mut visitor = TypeVisitor {
                types: Vec::new(),
                current_module: modules,
            };

            visitor.visit_file(&file_ast);
            all_types.extend(visitor.types);
        }
    }

    let mut registration_calls = String::new();
    // Sort types alphabetically for consistent output
    all_types.sort();
    for type_path in all_types {
        // Need to parse string to TokenStream or just append string
        registration_calls.push_str(&format!("    g.add_type::<{}>();\n", type_path));
    }

    let output_code = format!(
        "
        pub fn register_all_types(g: &mut facet_typescript::TypeScriptGenerator) {{
            {}
        }}
        ",
        registration_calls
    );

    fs::write(&dest_path, output_code).unwrap();
}
