# Kable Soundpack Format

Custom soundpacks allow you to personalize the launcher's audio experience with your own sounds and music.

## Structure

A soundpack can be either:
1. **A folder** containing `soundpack.json` and audio files
2. **A ZIP file** containing `soundpack.json` and audio files

Place soundpacks in: `%AppData%\.minecraft\kable\launcher\config\sounds\`

## soundpack.json Format

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
    "launch": "sounds/launch.mp3"
  },
  "music": {
    "menu": [
      "music/menu1.mp3",
      "music/menu2.mp3"
    ]
  }
}
```

## Sound Keys

Available sound effect keys:
- `click` - Button/UI clicks
- `hover` - UI hover sounds
- `success` - Success notifications
- `error` - Error notifications
- `notification` - General notifications
- `launch` - Game launch
- `install` - Mod/resource installation
- `delete` - Deletion operations
- `download` - Download complete
- `complete` - Task completion

## Music Playlists

Music is organized into playlists (e.g., "menu", "ambient"). Each playlist contains an array of music file paths that will play in sequence (with optional shuffle).

## Supported Formats

- MP3
- OGG
- WAV
- WebM (any format supported by Web Audio API)

## File Paths

All paths in `soundpack.json` are relative to the soundpack root:
- For folders: relative to the folder containing `soundpack.json`
- For ZIPs: relative to the ZIP root

## Example Structure

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
    └── ambient1.mp3
```

Or as a ZIP:
```
my-soundpack.zip
├── soundpack.json
├── sounds/
│   └── ... (audio files)
└── music/
    └── ... (music files)
```

## Tips

- Keep sound effects short (< 2 seconds) for better UX
- Use lower volumes for hover sounds to avoid fatigue
- Music tracks can be any length
- Compress audio files to reduce soundpack size
- Test all sounds before distributing
