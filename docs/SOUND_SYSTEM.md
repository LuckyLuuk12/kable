# Sound System Implementation

Complete sound system with configurable sound effects, background music, and custom soundpack support.

## Features

- ✅ Sound effects for UI interactions (click, hover, success, error, etc.)
- ✅ Background music with playlists
- ✅ Volume controls (master, sound effects, music)
- ✅ Custom soundpack support (folders or ZIP files)
- ✅ Settings UI integration
- ✅ Web Audio API-based playback
- ✅ Tauri backend for custom soundpack loading

## Architecture

### Frontend (TypeScript/Svelte)
- **SoundService.ts**: Main service managing sound playback
  - Web Audio API for high-quality audio
  - Sound effect pre-loading and caching
  - Music playlist management with shuffle/loop
  - Volume controls (master, sound, music)
  - Custom soundpack loading

- **soundActions.ts**: Svelte actions for easy sound integration
  - `buttonSound` - Generic button sound action
  - `successSound` - Success sound action
  - `errorSound` - Error sound action
  - `launchSound` - Launch sound action
  - Helper functions for direct sound playback

### Backend (Rust/Tauri)
- **commands/sounds.rs**: Backend commands for soundpack management
  - `list_soundpacks` - List available soundpacks
  - `get_soundpack_metadata` - Get soundpack configuration
  - `load_soundpack_file` - Load sound files from custom packs
  - `import_soundpack_zip` - Import new soundpacks

- **Soundpack Location**: `%AppData%\.minecraft\kable\launcher\config\sounds\`

## Usage

### 1. Using Sound Actions in Components

```svelte
<script>
  import { buttonSound, launchSound } from "$lib/services/soundActions";
</script>

<!-- Auto sound on click/hover -->
<button use:buttonSound>Click Me</button>

<!-- Custom sound -->
<button use:buttonSound={{ click: 'launch', hover: 'hover' }}>Launch Game</button>

<!-- Specific action sounds -->
<button use:launchSound>Launch</button>
```

### 2. Direct Sound Playback

```typescript
import { soundService } from "$lib/services/SoundService";

// Play a sound effect
soundService.playSound("click");
soundService.playSound("success", { volume: 0.8 });

// Play background music
soundService.playBackgroundMusic("menu", {
  shuffle: true,
  loop: true,
  volume: 0.5,
});

// Stop music
soundService.stopBackgroundMusic();
```

### 3. Managing Settings

```typescript
// Enable/disable sounds
soundService.setSoundEnabled(true);
soundService.setMusicEnabled(true);

// Adjust volumes (0-100)
soundService.setMasterVolume(80);
soundService.setSoundVolume(100);
soundService.setMusicVolume(50);

// Load a soundpack
await soundService.loadSoundpack("my-custom-pack");
```

## Soundpack Format

### Directory Structure

```
my-soundpack/
├── soundpack.json
├── sounds/
│   ├── click.mp3
│   ├── hover.mp3
│   ├── success.mp3
│   ├── error.mp3
│   ├── notification.mp3
│   └── launch.mp3
└── music/
    ├── menu1.mp3
    ├── menu2.mp3
    └── ambient.mp3
```

### soundpack.json

```json
{
  "name": "My Soundpack",
  "version": "1.0.0",
  "author": "Your Name",
  "sounds": {
    "click": "sounds/click.mp3",
    "hover": "sounds/hover.mp3",
    "success": "sounds/success.mp3",
    "error": "sounds/error.mp3",
    "notification": "sounds/notification.mp3",
    "launch": "sounds/launch.mp3",
    "install": "sounds/install.mp3",
    "delete": "sounds/delete.mp3"
  },
  "music": {
    "menu": ["music/menu1.mp3", "music/menu2.mp3"],
    "ambient": ["music/ambient.mp3"]
  }
}
```

### Supported Formats

- MP3
- OGG
- WAV
- WebM

### Available Sound Keys

- `click` - Button/UI clicks
- `hover` - UI hover sounds
- `success` - Success notifications
- `error` - Error notifications
- `notification` - General notifications
- `launch` - Game launch
- `install` - Installation complete
- `delete` - Deletion operations
- `download` - Download complete
- `complete` - Task completion

## Installation

### Default Soundpack

The default soundpack loads from `/static/sounds/`. To add default sounds:

1. Place audio files in `static/sounds/`
2. Files should match the keys: `click.mp3`, `hover.mp3`, `success.mp3`, etc.
3. Optional: Add `music/` folder for background music

### Custom Soundpacks

Users can install custom soundpacks by:

1. **Folder**: Place soundpack folder in `%AppData%\.minecraft\kable\launcher\config\sounds\`
2. **ZIP**: Place ZIP file in the same directory (must contain `soundpack.json` at root)
3. **UI Import**: Use the settings UI to import a ZIP file

## Configuration

Sound settings are stored in `AppearanceSettings`:

```typescript
interface SoundSettings {
  enabled: boolean;           // Enable sound effects
  music_enabled: boolean;     // Enable background music
  master_volume: number;      // 0-100
  sound_volume: number;       // 0-100
  music_volume: number;       // 0-100
  selected_soundpack: string; // "default" or custom pack name
}
```

## Settings UI

Sound controls are integrated into the Appearance settings tab:

- ✅ Enable/disable sound effects
- ✅ Enable/disable background music
- ✅ Master volume slider
- ✅ Sound effects volume slider
- ✅ Music volume slider
- ✅ Soundpack selection dropdown

## Initialization

The sound system initializes automatically on app startup in `+layout.svelte`:

```typescript
onMount(async () => {
  // Initialize sound service
  await soundService.initialize();
});
```

## Best Practices

1. **Keep sound effects short** (< 2 seconds) for better UX
2. **Use lower volumes for hover sounds** (0.3) to avoid fatigue
3. **Pre-load sound effects** but stream music to save memory
4. **Compress audio files** to reduce soundpack size
5. **Test all sounds** before distributing custom packs
6. **Respect licenses** when using third-party sounds

## Example: Adding Sounds to Existing Buttons

```svelte
<script>
  import { buttonSound, successSound } from "$lib/services/soundActions";
</script>

<!-- Basic button with sound -->
<button use:buttonSound on:click={doSomething}>
  Do Something
</button>

<!-- Success button -->
<button use:successSound on:click={saveData}>
  Save
</button>

<!-- Custom sound -->
<button use:buttonSound={{ click: 'launch' }} on:click={launchGame}>
  Launch Game
</button>
```

## File Locations

- **Frontend Service**: `src/lib/services/SoundService.ts`
- **Sound Actions**: `src/lib/services/soundActions.ts`
- **Backend Commands**: `src-tauri/src/commands/sounds.rs`
- **Settings Integration**: `src/lib/components/settings/AppearanceSettingsUI.svelte`
- **Default Sounds**: `static/sounds/`
- **Custom Soundpacks**: `%AppData%\.minecraft\kable\launcher\config\sounds\`
- **Example Soundpack**: `examples/soundpack-example/`

## Future Enhancements

- [ ] Soundpack preview in settings
- [ ] Per-action sound customization
- [ ] Audio spectrum visualizer
- [ ] Sound effect mixing/layering
- [ ] Fade in/out transitions
- [ ] Spatial audio support
- [ ] Voice line support
- [ ] Achievement sounds
- [ ] Notification sound categories
