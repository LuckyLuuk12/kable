// Include the generated registration function
include!(concat!(env!("OUT_DIR"), "/register_types.rs"));

fn generate_typescript_types() -> String {
    let mut output = String::new();

    // Generate TypeScript types for API types
    output.push_str("// This file is auto-generated with `cd api-typegen && cargo run`. Do not edit directly.\n\n");

    let mut g = facet_typescript::TypeScriptGenerator::new();

    // Automatically register all types found in api-types
    register_all_types(&mut g);

    output.push_str(&g.finish());
    output
}

/**
 * Run this with `cargo run` after `cd api-typegen` to generate the TypeScript types and write them to `src/lib/api-types.ts`.
 * You can also run `cargo run --bin api-typegen check` to only check if the generated code matches the existing code without writing to the file system.
 */
fn main() {
    // If we have "check" as the first argument, we only want to check if the generated code is the same as the existing code, without actually writing to the file system.
    let check_only = std::env::args().nth(1).as_deref() == Some("check");

    let generated_code_path = std::path::Path::new("../src/lib/api-types.ts");

    let generated_code = generate_typescript_types();

    if check_only {
        let existing_code = std::fs::read_to_string(generated_code_path).expect("Failed to read existing generated code");
        // Normalize line endings for comparison
        let existing_code = existing_code.replace("\r\n", "\n");
        let generated_code = generated_code.replace("\r\n", "\n");
        if existing_code != generated_code {
            eprintln!(
                "Generated code does not match existing code. Please run `cd api-typegen && cargo run` to update the generated code."
            );
            dbg!(&existing_code);
            dbg!(&generated_code);
            for (i, (existing_line, generated_line)) in existing_code.lines().zip(generated_code.lines()).enumerate() {
                if existing_line != generated_line {
                    eprintln!("Difference at line {}:\nExisting: {}\nGenerated: {}", i + 1, existing_line, generated_line);
                }
            }
            std::process::exit(1);
        } else {
            println!("Generated code matches existing code.");
        }
    } else {
        std::fs::write(generated_code_path, generated_code).expect("Failed to write generated code to file");
        println!("Generated code written to {}", generated_code_path.display());
    }
}
