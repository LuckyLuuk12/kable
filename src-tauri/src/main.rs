// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  // Load environment variables from .env file
  dotenvy::dotenv().ok();
  
  app_lib::run();
}
