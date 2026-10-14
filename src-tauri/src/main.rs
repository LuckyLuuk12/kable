// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Generate TypeScript types for the API commands using Specta
    #[cfg(debug_assertions)]
    app_lib::typegen::generate().expect("specta failed");

    app_lib::run();
}
