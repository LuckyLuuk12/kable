# Kable Minecraft Launcher Setup

## Overview

You've successfully set up a Minecraft launcher with the following core features:

### ✅ Completed Features

1. **Microsoft OAuth2 Authentication**
   - Full OAuth2 flow with Microsoft accounts
   - Xbox Live authentication integration
   - Minecraft profile fetching
   - Token refresh functionality
   - Multiple account support

2. **Settings Management**
   - Persistent settings storage in `%APPDATA%/kable-launcher/settings.json`
   - Cross-platform directory detection
   - Type-safe settings with TypeScript interfaces matching Rust structs

3. **Minecraft Installation Detection**
   - Auto-discovery of Minecraft installations
   - Cross-platform support (Windows, macOS, Linux)
   - Version detection and validation
   - Support for vanilla and modded installations

4. **Cached Username Support**
   - Reads from Minecraft's `usercache.json` for offline reference
   - Useful for testing and development

### 🏗️ Architecture

**Backend (Rust/Tauri):**
- `src-tauri/src/lib.rs` - Main Tauri commands and OAuth implementation
- Persistent settings stored in system data directory
- Full Microsoft/Xbox Live/Minecraft authentication chain

**Frontend (Svelte/TypeScript):**
- `src/lib/types.ts` - Shared type definitions
- `src/lib/services.ts` - Service layer for Tauri command invocation

### 📁 File Structure

```
Settings Location: %APPDATA%/kable-launcher/
└── settings.json (stores accounts, preferences, etc.)

Minecraft Detection:
├── Windows: %APPDATA%/.minecraft
├── macOS: ~/Library/Application Support/minecraft  
└── Linux: ~/.minecraft
```

### 🔧 Next Steps

**To set up Microsoft OAuth:**
1. Register your app at https://portal.azure.com/
2. Replace `YOUR_MICROSOFT_CLIENT_ID` in `lib.rs` with your actual client ID
3. Configure redirect URI: `http://localhost:8080/callback`

**To implement version selection:**
- Add Minecraft version manifest fetching
- Implement mod loader detection
- Add version installation

**To add mod/modpack support:**
- Integrate Modrinth API
- Integrate CurseForge API  
- Implement mod version management

### 🚀 Running the Launcher

```bash
npm run tauri dev  # Development mode
npm run tauri build  # Production build
```

The launcher will create its data directory automatically on first run and store all settings persistently across sessions.
