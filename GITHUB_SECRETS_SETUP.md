# Quick GitHub Secrets Setup

## Step 1: Get Your Private Key Content

Run this command to copy your private key content:

```bash
cat C:\Users\luukk\.tauri\kable.key
```

Copy the entire output (it will be a long base64-like string).

## Step 2: Add GitHub Secrets

1. Go to: https://github.com/LuckyLuuk12/kable/settings/secrets/actions

2. Click **"New repository secret"**

3. Add these two secrets:

### Secret 1: `TAURI_SIGNING_PRIVATE_KEY`
- **Name**: `TAURI_SIGNING_PRIVATE_KEY`  
- **Value**: Paste the entire content from the `cat` command above

### Secret 2: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- **Name**: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- **Value**: The password you entered when generating the key

## Step 3: Test the Workflow

```bash
# Create a test release
git checkout -b release
git push origin release
```

The GitHub Action will automatically:
- Build signed installers for all platforms
- Generate the `latest.json` file
- Create a GitHub release with all artifacts

## Step 4: Test Auto-Update

After the release is created:
1. Go to **Settings → Misc → Auto-Update** in your app
2. Click "Check for Updates"
3. Should find the new release from GitHub

That's it! The auto-update system will now work automatically. 🎉
