# WiX Installer Customization

Kable uses a custom WiX installer template for Windows MSI packages with dark theme styling and branding.

## Setup

The custom WiX configuration is located in `src-tauri/wix/`:

- `main.wxs` - Main WiX template with custom UI
- `lang/` - Localization files (en-US.wxl, nl-NL.wxl)
- `licenses/` - End User License Agreements (en-US, nl-NL)
- `img/` - Installer images (banner.bmp, dialog.bmp)

## Features

### Custom Branding
- Dark theme matching Kable's design (background: `#0a0a0f`)
- Purple accent color (`#8b5cf6`) throughout the installer
- Professional welcome dialog with project information

### Built-in Links
The installer includes clickable links to:
- **Official Website**: https://kable.kablan.nl
- **Documentation Wiki**: https://github.com/LuckyLuuk12/kable/wiki
- **Support Development**: https://ko-fi.com/luckyluuk

### User-Friendly
- Custom welcome screen with project description
- Installation directory selection
- Desktop shortcut option
- Start menu integration
- Links in Add/Remove Programs

## Generating Images

To regenerate the banner and dialog images with the theme colors:

```bash
# Install Pillow if needed
pip install pillow

# Generate images
npm run generate:wix-images
```

This creates `banner.bmp` and `dialog.bmp` in `src-tauri/wix/` using the project's theme colors from `src/lib/styles/global.scss`.

## Building the Installer

The WiX configuration is automatically used when building for Windows:

```bash
npm run tauri build
```

The MSI installer will be created in `src-tauri/target/release/bundle/msi/`.

## Customization

### Colors
Theme colors are defined in the Python script and SCSS:
- Primary: `#8b5cf6` (purple)
- Secondary: `#ec4899` (pink)
- Tertiary: `#0ea5e9` (cyan)
- Background: `#0a0a0f` (dark)

### License
Edit `src-tauri/wix/licenses/License_en-US.rtf` or `License_nl-NL.rtf` to modify the EULA shown during installation.

### Images
Run `npm run generate:wix-images` or manually edit:
- `img/banner.bmp` - 493 × 58 pixels (top banner)
- `img/dialog.bmp` - 164 × 314 pixels (left sidebar)

Both must be in 24-bit BMP format.

### UI Text
Modify `src-tauri/wix/main.wxs` to change:
- Welcome message
- Dialog titles and descriptions
- Button labels (uses WiX localization)

## References

- [Tauri v2 Windows Configuration](https://v2.tauri.app/reference/config/#wixconfig)
- [Tauri v1 WiX Guide](https://v1.tauri.app/v1/guides/building/windows/#customizing-the-wix-installer-template)
- [WiX Toolset Documentation](https://wixtoolset.org/docs/)

## Notes

- WiX template is only used for Windows MSI builds
- Images must be BMP format (not PNG or JPEG)
- The installer automatically detects and uses WebView2
- Elevated privileges are supported for updates
- The UpgradeCode should remain consistent across versions for proper upgrade detection

## Troubleshooting

### "License.rtf not found"
Ensure `src-tauri/wix/licenses/License_en-US.rtf` and `License_nl-NL.rtf` exist and are in RTF format.

### "Invalid BMP format"
Banner and dialog images must be 24-bit BMP format. Use the Python script to generate them or convert with an image editor.

### "Template not found"
Verify `template: "wix/main.wxs"` in `src-tauri/tauri.conf.json` points to the correct path relative to `src-tauri/`.

### Build errors
Run `npm run tauri build -- --verbose` for detailed WiX compiler output.
