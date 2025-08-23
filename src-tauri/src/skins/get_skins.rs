use crate::auth::{get_minecraft_account, AuthMethod, LauncherAccount};
use crate::logging::{LogLevel, Logger};
use crate::skins::types::{CurrentSkin, SkinModel, AccountSkin};
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;
use std::fs;
use crate::skins::types::{PlayerProfile, AccountCape};


/// Get the full player profile (id, name, skins, capes) from Mojang API
pub async fn get_player_profile() -> Result<PlayerProfile, String> {
	Logger::console_log(
		LogLevel::Info,
		"🔍 Fetching player profile (skins & capes)",
		None,
	);

	// Get the authenticated account
	let account = get_minecraft_account(Some(AuthMethod::DeviceCodeFlow))
		.await
		.map_err(|e| format!("Authentication required: {}", e))?;

	let client = reqwest::Client::new();
	let url = "https://api.minecraftservices.com/minecraft/profile";
	let response = client
		.get(url)
		.header("Authorization", format!("Bearer {}", account.access_token))
		.send()
		.await
		.map_err(|e| format!("Failed to fetch profile: {}", e))?;

	if response.status() != reqwest::StatusCode::OK {
		return Err(format!("Profile request failed with status: {}", response.status()));
	}

	let profile_data: serde_json::Value = response
		.json()
		.await
		.map_err(|e| format!("Failed to parse profile response: {}", e))?;

	let id = profile_data.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
	let name = profile_data.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();

	// Parse skins
	let mut skins = Vec::new();
	if let Some(skins_arr) = profile_data.get("skins").and_then(|v| v.as_array()) {
		for skin in skins_arr {
			let id = skin.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
			let name = "Account Skin".to_string();
			let url = skin.get("url").and_then(|v| v.as_str()).map(|s| s.to_string());
			let variant = skin.get("variant").and_then(|v| v.as_str()).unwrap_or("CLASSIC");
			let model = match variant {
				"SLIM" => SkinModel::Slim,
				_ => SkinModel::Classic,
			};
			let is_current = skin.get("state").and_then(|v| v.as_str()) == Some("ACTIVE");
			skins.push(AccountSkin {
				id,
				name,
				url,
				model,
				is_current,
				uploaded_date: None,
			});
		}
	}

	// Parse capes
	let mut capes = Vec::new();
	if let Some(capes_arr) = profile_data.get("capes").and_then(|v| v.as_array()) {
		for cape in capes_arr {
			let id = cape.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
			let state = cape.get("state").and_then(|v| v.as_str()).unwrap_or_default().to_string();
			let url = cape.get("url").and_then(|v| v.as_str()).map(|s| s.to_string());
			let alias = cape.get("alias").and_then(|v| v.as_str()).map(|s| s.to_string());
			capes.push(AccountCape {
				id,
				state,
				url,
				alias,
			});
		}
	}

	Ok(PlayerProfile {
		id,
		name,
		skins,
		capes,
	})
}


/// Get the current skin information from Mojang
pub async fn get_current_skin_info() -> Result<CurrentSkin, String> {
	Logger::console_log(
		LogLevel::Info,
		"🔍 Fetching current skin information",
		None,
	);

	// Get the authenticated account
	let account = get_minecraft_account(Some(AuthMethod::DeviceCodeFlow))
		.await
		.map_err(|e| format!("Authentication required: {}", e))?;

	get_current_skin(&account).await
}

/// Get all skins stored in the user's Microsoft/Mojang account and local Minecraft directory
pub async fn get_all_account_skins() -> Result<Vec<AccountSkin>, String> {
	Logger::console_log(
		LogLevel::Info,
		"🎨 Fetching all skins from Microsoft account and local directory",
		None,
	);

	let mut all_skins = Vec::new();

	// Get the current skin from Mojang API
	match get_minecraft_account(Some(AuthMethod::DeviceCodeFlow)).await {
		Ok(account) => {
			match get_current_skin(&account).await {
				Ok(current_skin) => {
					if current_skin.has_skin {
						let account_skin = AccountSkin {
							id: "current".to_string(),
							name: "Current Skin".to_string(),
							url: current_skin.url,
							model: current_skin.model,
							is_current: true,
							uploaded_date: None, // We don't have this information from the API
						};
						all_skins.push(account_skin);
					}
				}
				Err(e) => {
					Logger::console_log(
						LogLevel::Warning,
						&format!("Failed to fetch current skin: {}", e),
						None,
					);
				}
			}
		}
		Err(e) => {
			Logger::console_log(
				LogLevel::Warning,
				&format!("Authentication not available, skipping online skins: {}", e),
				None,
			);
		}
	}

	// Get local skins from Minecraft directory
	match get_local_skins().await {
		Ok(local_skins) => {
			all_skins.extend(local_skins);
		}
		Err(e) => {
			Logger::console_log(
				LogLevel::Warning,
				&format!("Failed to load local skins: {}", e),
				None,
			);
		}
	}

	Logger::console_log(
		LogLevel::Info,
		&format!("📊 Found {} total skins ({} online, {} local)", 
				all_skins.len(),
				all_skins.iter().filter(|s| s.id == "current").count(),
				all_skins.iter().filter(|s| s.id != "current").count()),
		None,
	);

	Ok(all_skins)
}

/// Get all local skins from the Minecraft directory
pub async fn get_local_skins() -> Result<Vec<AccountSkin>, String> {
	let minecraft_dir = crate::get_default_minecraft_dir()?;
	let custom_skins_file = minecraft_dir.join("launcher_custom_skins.json");

	if !custom_skins_file.exists() {
		Logger::console_log(
			LogLevel::Debug,
			&format!("📁 No custom skins file found at: {}", custom_skins_file.display()),
			None,
		);
		return Ok(Vec::new());
	}

	Logger::console_log(
		LogLevel::Debug,
		&format!("📁 Reading custom skins from: {}", custom_skins_file.display()),
		None,
	);

	let file_contents = fs::read_to_string(&custom_skins_file)
		.map_err(|e| format!("Failed to read launcher_custom_skins.json: {}", e))?;

	let root: crate::CustomSkinsRoot = serde_json::from_str(&file_contents)
		.map_err(|e| format!("Failed to parse launcher_custom_skins.json: {}", e))?;

	let mut local_skins = Vec::new();

	for (key, skin_entry) in root.custom_skins {
		match convert_custom_skin_to_account_skin(key, skin_entry).await {
			Ok(skin) => {
				local_skins.push(skin);
			}
			Err(e) => {
				Logger::console_log(
					LogLevel::Warning,
					&format!("Failed to convert skin entry: {}", e),
					None,
				);
			}
		}
	}

	Logger::console_log(
		LogLevel::Debug,
		&format!("📄 Found {} local skins in launcher_custom_skins.json", local_skins.len()),
		None,
	);

	Ok(local_skins)
}

/// Convert a CustomSkinEntry to an AccountSkin
pub async fn convert_custom_skin_to_account_skin(key: String, skin_entry: super::CustomSkinEntry) -> Result<AccountSkin, String> {
	// Determine model from 'slim' field
	let model = if skin_entry.slim { SkinModel::Slim } else { SkinModel::Classic };

	// Use the updated or created date if available
	let uploaded_date = None;

	// Helper: check if string is likely base64 PNG
	fn is_base64_png(data: &str) -> bool {
		let trimmed = data.trim();
		trimmed.starts_with("data:image/png;base64,") ||
		(trimmed.len() > 100 && trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='))
	}

	// Try to construct a file path from skinImage or modelImage
	let mut found_url = None;
	let tried_base64 = false;

	for (img_data, img_type) in [(&skin_entry.skin_image, "skin"), (&skin_entry.model_image, "model")] {
		if img_data.trim().is_empty() { continue; }

		if is_base64_png(img_data) {
			// Remove data URI prefix if present
			let b64 = if img_data.trim().starts_with("data:image/png;base64,") {
				&img_data.trim()["data:image/png;base64,".len()..]
			} else {
				img_data.trim()
			};
			// Always return a data URL
			found_url = Some(format!("data:image/png;base64,{}", b64));
			Logger::console_log(
				LogLevel::Debug,
				&format!("Returning data URL for skin '{}', type '{}', length {}", key, img_type, b64.len()),
				None,
			);
			break;
		} else {
			// Try as file path in Minecraft dir
			let minecraft_dir = crate::get_default_minecraft_dir()
				.map_err(|e| format!("Failed to get minecraft directory: {}", e))?;
			let candidate = minecraft_dir.join(img_data.trim());
			if candidate.exists() {
				match std::fs::read(&candidate) {
					Ok(file_bytes) => {
						let b64 = general_purpose::STANDARD.encode(&file_bytes);
						found_url = Some(format!("data:image/png;base64,{}", b64));
						Logger::console_log(
							LogLevel::Debug,
							&format!("Read skin image file for skin '{}', type '{}', returned as data URL, length {}", key, img_type, b64.len()),
							None,
						);
						break;
					}
					Err(e) => {
						Logger::console_log(
							LogLevel::Warning,
							&format!("Failed to read skin image file for skin '{}': {}", key, e),
							None,
						);
					}
				}
			}
		}
	}

	if found_url.is_none() && tried_base64 {
		Logger::console_log(
			LogLevel::Warning,
			&format!("Failed to decode base64 skin image for skin '{}'.", key),
			None,
		);
	}

	Ok(AccountSkin {
		id: format!("local_{}", key),
		name: skin_entry.name,
		url: found_url,
		model,
		is_current: false,
		uploaded_date,
	})
}

/// Get current skin information from Mojang profile API
pub async fn get_current_skin(account: &LauncherAccount) -> Result<CurrentSkin, String> {
	let client = reqwest::Client::new();
    
	// Get profile with textures
	let url = format!(
		"https://sessionserver.mojang.com/session/minecraft/profile/{}",
		account.minecraft_profile.id
	);

	Logger::console_log(
		LogLevel::Debug,
		&format!("🌐 Fetching profile from: {}", url),
		None,
	);

	let response = client
		.get(&url)
		.send()
		.await
		.map_err(|e| format!("Failed to fetch profile: {}", e))?;

	if response.status() != reqwest::StatusCode::OK {
		return Err(format!("Profile request failed with status: {}", response.status()));
	}

	let profile_data: Value = response
		.json()
		.await
		.map_err(|e| format!("Failed to parse profile response: {}", e))?;

	// Parse the profile data to extract skin information
	let properties = profile_data
		.get("properties")
		.and_then(|p| p.as_array())
		.ok_or_else(|| "No properties found in profile".to_string())?;

	for property in properties {
		if property.get("name").and_then(|n| n.as_str()) == Some("textures") {
			let value = property
				.get("value")
				.and_then(|v| v.as_str())
				.ok_or_else(|| "No texture value found".to_string())?;

			// Decode base64
			let decoded = general_purpose::STANDARD
				.decode(value)
				.map_err(|e| format!("Failed to decode texture data: {}", e))?;

			let texture_data: Value = serde_json::from_slice(&decoded)
				.map_err(|e| format!("Failed to parse texture JSON: {}", e))?;

			// Extract skin information
			if let Some(skin) = texture_data.get("textures").and_then(|t| t.get("SKIN")) {
				let url = skin.get("url").and_then(|u| u.as_str()).map(|s| s.to_string());
				let metadata = skin.get("metadata");
                
				let model = if let Some(meta) = metadata {
					let model_str = meta.get("model").and_then(|m| m.as_str()).unwrap_or("classic");
					SkinModel::from_api_string(model_str).unwrap_or(SkinModel::Classic)
				} else {
					SkinModel::Classic
				};

				return Ok(CurrentSkin {
					model,
					url,
					has_skin: true,
				});
			}
		}
	}

	Ok(CurrentSkin {
		model: SkinModel::Classic,
		url: None,
		has_skin: false,
	})
}

/// Download skin data from a URL
pub async fn download_skin_from_url(url: &str) -> Result<Vec<u8>, String> {
	let client = reqwest::Client::new();
    
	Logger::console_log(
		LogLevel::Debug,
		&format!("⬇️ Downloading skin from: {}", url),
		None,
	);

	let response = client
		.get(url)
		.send()
		.await
		.map_err(|e| format!("Failed to download skin: {}", e))?;

	if response.status() != reqwest::StatusCode::OK {
		return Err(format!("Download failed with status: {}", response.status()));
	}

	let data = response
		.bytes()
		.await
		.map_err(|e| format!("Failed to read skin data: {}", e))?;

	Ok(data.to_vec())
}
