# WiX Installer Assets

This folder contains custom WiX installer configuration for Kable.

## Files

- `main.wxs` - Main WiX template with custom UI and branding
- `lang/en-US.wxl` - English localization strings (referenced in tauri.conf.json)
- `lang/nl-NL.wxl` - Dutch localization strings (referenced in tauri.conf.json)
- `licenses/License_en-US.rtf` - End User License Agreement (English)
- `licenses/License_nl-NL.rtf` - End User License Agreement (Dutch)
- `img/banner.bmp` - Top banner image (493 × 58 pixels)
- `img/dialog.bmp` - Left sidebar image (164 × 314 pixels)

**Note**: Localization files in `lang/` are referenced via `localePath` in `tauri.conf.json`.

## Creating Banner and Dialog Images

### Banner (493 × 58 pixels)
Create a banner with Kable branding using theme colors:
- Background: #0a0a0f (dark)
- Primary accent: #8b5cf6 (purple)
- Text: "Kable - Minecraft Launcher"

### Dialog (164 × 314 pixels)
Create a vertical left sidebar image with:
- Background gradient from #0a0a0f to #1e1e20
- Vertical "KABLE" branding
- Subtle purple accent (#8b5cf6)

## Theme Colors Used
- Primary: #8b5cf6 (purple)
- Secondary: #ec4899 (pink)
- Tertiary: #0ea5e9 (cyan)
- Background: #0a0a0f (dark)
- Container: #1e1e20

## Links Included
- Official Website: https://kable.kablan.nl
- Documentation Wiki: https://github.com/LuckyLuuk12/kable/wiki
- Support (Ko-fi): https://ko-fi.com/luckyluuk
