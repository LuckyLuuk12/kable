# WiX Installer Customization - Summary

✓ **Custom WiX installer successfully configured for Kable**

## What Was Created

### Configuration Files
- [src-tauri/wix/main.wxs](../src-tauri/wix/main.wxs) - Custom WiX template with dark theme
- [src-tauri/wix/License.rtf](../src-tauri/wix/License.rtf) - End User License Agreement
- [src-tauri/wix/README.md](../src-tauri/wix/README.md) - WiX folder documentation

### Generated Assets
- [src-tauri/wix/banner.bmp](../src-tauri/wix/banner.bmp) - Top banner (493 × 58 px) ✓
- [src-tauri/wix/dialog.bmp](../src-tauri/wix/dialog.bmp) - Side panel (493 × 312 px) ✓

### Scripts & Documentation
- [scripts/generate-wix-images.py](../scripts/generate-wix-images.py) - Image generator script
- [docs/WIX_INSTALLER.md](../docs/WIX_INSTALLER.md) - Complete guide
- Updated [package.json](../package.json) - Added `generate:wix-images` script
- Updated [src-tauri/tauri.conf.json](../src-tauri/tauri.conf.json) - WiX configuration

## Features Implemented

### 🎨 Dark Theme Styling
- Background: `#0a0a0f` (matches Kable's dark theme)
- Primary accent: `#8b5cf6` (purple)
- Professional gradient effects
- Consistent with your design system

### 🔗 Integrated Links
All three requested links are included in the welcome dialog:
1. **Official Website**: https://kable.kablan.nl
2. **Documentation Wiki**: https://github.com/LuckyLuuk12/kable/wiki
3. **Support Development**: https://ko-fi.com/luckyluuk

### 📦 Enhanced User Experience
- Custom welcome screen with project description
- Installation directory selection
- Desktop shortcut option
- Start menu shortcuts
- Links in Windows Add/Remove Programs
- Professional branding throughout

## Theme Colors Used

```scss
Primary:     #8b5cf6  // Purple (from global.scss)
Secondary:   #ec4899  // Pink
Tertiary:    #0ea5e9  // Cyan
Background:  #0a0a0f  // Dark
Container:   #1e1e20  // Dark lighter
Text:        #ffffff  // White
Placeholder: #a0a0aa  // Light gray
```

## Quick Commands

```bash
# Generate/regenerate installer images
npm run generate:wix-images

# Build Windows MSI with custom installer
npm run tauri build

# Test installer (after build)
# Output: src-tauri/target/release/bundle/msi/Kable_0.1.8_x64_en-US.msi
```

## How It Works

1. **Template**: `main.wxs` defines the installer UI, dialogs, and components
2. **Images**: `banner.bmp` and `dialog.bmp` provide visual branding
3. **License**: `License.rtf` shows EULA with clickable links
4. **Config**: `tauri.conf.json` references the WiX template
5. **Build**: Tauri automatically uses the custom template when building MSI

## Customization

To modify the installer:

### Change Colors
Edit `scripts/generate-wix-images.py` and run:
```bash
npm run generate:wix-images
```

### Update License Text
Edit `src-tauri/wix/License.rtf` (RTF format required)

### Modify UI Text/Layout
Edit `src-tauri/wix/main.wxs` (WiX XML format)

### Add More Links
Add `<Control Id="..." Type="Hyperlink" ...>` elements in `main.wxs`

## Next Steps

1. ✓ Configuration complete
2. ✓ Images generated with theme colors
3. ✓ Links integrated (Ko-fi, Wiki, Website)
4. **Test**: Run `npm run tauri build` to create the MSI
5. **Verify**: Install the MSI and check the installer appearance
6. **Iterate**: Adjust colors/text in `main.wxs` or regenerate images as needed

## References

- [Tauri v2 WiX Config](https://v2.tauri.app/reference/config/#wixconfig)
- [WiX Toolset Docs](https://wixtoolset.org/docs/)
- [Complete Guide](WIX_INSTALLER.md)

---

**Note**: The WiX customization only applies to Windows MSI builds. Other platforms (macOS, Linux) use their respective packaging systems.
