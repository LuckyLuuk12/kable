# Auto-Update Setup

This document explains how the auto-update system works and how to set it up for releases.

## Key Generation

The signing keys have already been generated using:

```bash
npx tauri signer generate -w C:\Users\luukk\.tauri\kable.key
```

This created:
- `C:\Users\luukk\.tauri\kable.key` - **Private key** (keep secret!)
- `C:\Users\luukk\.tauri\kable.key.pub` - **Public key** (already in tauri.conf.json)

## Environment Variables for Building

When building releases that support auto-updates, you need to set these environment variables:

### Windows (PowerShell)
```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = "C:\Users\luukk\.tauri\kable.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "your_password_here"
```

### Windows (Command Prompt)
```cmd
set TAURI_SIGNING_PRIVATE_KEY=C:\Users\luukk\.tauri\kable.key
set TAURI_SIGNING_PRIVATE_KEY_PASSWORD=your_password_here
```

### Linux/macOS
```bash
export TAURI_SIGNING_PRIVATE_KEY="C:\Users\luukk\.tauri\kable.key"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="your_password_here"
```

## Building with Updates

To build a release with update artifacts:

```bash
# Set environment variables first (see above)
npm run tauri build
```

This will create:
- Regular installers (MSI, NSIS, etc.)
- Update artifacts (.sig signature files)
- Update metadata

## GitHub Releases Integration

The app checks for updates at:
```
https://github.com/LuckyLuuk12/kable/releases/latest/download/latest.json
```

### GitHub Secrets Setup

Before the workflow can build signed releases, you need to add these secrets to your GitHub repository:

1. Go to your GitHub repo → **Settings** → **Secrets and variables** → **Actions**
2. Add these **Repository secrets**:

   - **`TAURI_SIGNING_PRIVATE_KEY`**: 
     - Copy the content of your private key file (`C:\Users\luukk\.tauri\kable.key`)
     - You can read it with: `cat C:\Users\luukk\.tauri\kable.key`
   
   - **`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`**: 
     - The password you set when generating the key

### Automatic Updates with GitHub Actions

The updated workflow (`.github/workflows/main.yml`) now:

1. **Uses Tauri Action** with `includeUpdaterJson: true`
2. **Sets signing environment variables** from GitHub secrets  
3. **Automatically generates** the `latest.json` file for updates
4. **Creates signed installers** and update artifacts
5. **Publishes everything** to GitHub releases

### Triggering a Release

1. **Push to `release` branch**:
   ```bash
   git checkout -b release
   git push origin release
   ```

2. **Or create a release manually** in GitHub UI

The action will:
- Build for all platforms (Windows, macOS, Linux)
- Sign the installers with your private key
- Generate the `latest.json` updater manifest
- Upload everything to the GitHub release

## Testing the Update System

### 1. First Release
```bash
# Make sure secrets are set in GitHub
# Push to release branch
git checkout -b release
git push origin release
```

### 2. Check Generated Files
After the action completes, verify the release contains:
- Platform installers (`.msi`, `.dmg`, `.AppImage`)
- Signature files (`.sig`)
- **`latest.json`** file (this is what the app checks)

### 3. Test Update Check
1. Install an older version of your app
2. Run the newer version in dev mode
3. Go to **Settings → Misc → Auto-Update**
4. Click "Check for Updates"
5. Should detect the newer version from GitHub

## Troubleshooting

### Common Issues:

1. **"No update available"** but there should be:
   - Check if `latest.json` exists in the latest release
   - Verify the endpoint URL in `tauri.conf.json`
   - Ensure version number is higher than current

2. **"Failed to check for updates"**:
   - Check network connectivity
   - Verify GitHub repository is public or token has access
   - Check browser console for error details

3. **"Failed to install update"**:
   - Signature verification failed - check private key matches public key
   - Download failed - verify installer URLs in `latest.json`
   - Permissions issue - app may need admin rights on Windows

### Debug Mode
For detailed logs, check the browser console in dev mode when testing updates.

## Security Notes

⚠️ **CRITICAL**: 
- **Never commit** the private key to version control
- **Never share** the private key with anyone
- **Back up** the private key securely - if lost, you cannot publish updates
- The private key password protects the key file

## How It Works

1. Users can check for updates in **Settings → Misc → Auto-Update**
2. The app queries GitHub releases for new versions
3. If an update is available, users can install it with one click
4. The app downloads, verifies (using the signature), and installs the update
5. The app automatically restarts with the new version

## Configuration

The updater configuration is in `src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDJDMDE4NEJFREU1MTAyOUEKUldTYUFsSGV2b1FCTEtFaWREK2xiMnlDU21xTStYNnIzcWltTzR1RjA4Ni8zaGFNRzhRSm5hVzAK",
      "endpoints": [
        "https://github.com/LuckyLuuk12/kable/releases/latest/download/latest.json"
      ]
    }
  },
  "bundle": {
    "createUpdaterArtifacts": true
  }
}
```

Permissions are configured in `src-tauri/capabilities/default.json`:
```json
{
  "permissions": [
    "updater:default"
  ]
}
```
