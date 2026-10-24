# Kable
### An Unofficial Minecraft Launcher

Kable is a customizable, developer-friendly Minecraft launcher built with **Tauri, Svelte, and Rust**. Designed for modders and power users, it combines mod management, installation profiles, detailed logging, and extensive customization in one desktop application.

Rather than hiding complexity, Kable gives you the tools to understand, configure, and manage your Minecraft installations on your own terms.

[![wakatime](https://wakatime.com/badge/user/8521d65d-1c6b-41ba-b570-372359980d5f/project/b62f37c7-03f9-4bc5-849e-307f65bd8b3d.svg?style=flat-square&color=5a20f2)](https://wakatime.com/badge/user/8521d65d-1c6b-41ba-b570-372359980d5f/project/b62f37c7-03f9-4bc5-849e-307f65bd8b3d)
![Latest Release](https://img.shields.io/github/v/release/LuckyLuuk12/kable?style=flat-square&color=blue)
![Downloads](https://img.shields.io/github/downloads/LuckyLuuk12/kable/total?style=flat-square&color=green)
![License](https://img.shields.io/badge/license-Proprietary-red.svg?style=flat-square)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg?style=flat-square)
![GitHub Stars](https://img.shields.io/github/stars/LuckyLuuk12/kable?style=flat-square&color=yellow)
![Issues](https://img.shields.io/github/issues/LuckyLuuk12/kable?style=flat-square&color=orange)

## ✨ Features

### Modding Made Easier
Browse, install, and manage mods, resource packs, and shader packs directly from Kable. Automatic dependency resolution helps ensure that required projects are installed and compatible with your Minecraft installation.

### Powerful Profile Management
Import and export profiles, merge existing installations, manage their contents, and create shortcuts for quick access. Advanced merging capabilities help bring different configurations together without requiring you to rebuild everything manually.

### Advanced Logging & Debugging
Inspect Minecraft launch output and application logs through an integrated logs panel. Smart filtering, fuzzy search, and other tools make it easier to find relevant information and diagnose problems without digging through log files manually.

### Skins & Worlds
Manage your Minecraft skins and worlds directly within Kable, keeping important parts of your Minecraft experience accessible from one place.

### Extensive Customization
Make Kable your own with custom CSS, icons, themes, and other configuration options. Fine-tune the launcher to suit your workflow instead of being limited to a fixed interface.

### More Than a Launcher
Kable combines installation management, modding tools, diagnostics, and customization in a single application. Additional functionality is available throughout the interface for users who want more control over their Minecraft setup.

## 🚀 Getting Started

### Prerequisites

Before installing Kable, make sure you have:

- **An existing Minecraft Java Edition installation.** You must have launched Minecraft at least once through the official Minecraft Launcher on your system so that your `.minecraft` directory exists.
- **A valid Minecraft account** with access to Java Edition.
- **A supported operating system:** Windows 10 or later, macOS, or a compatible Linux distribution.

Kable manages your existing Minecraft environment. It is not intended to replace the initial setup performed by the official launcher.

### Installation

1. If you haven't already, install the [official Minecraft Launcher](https://www.minecraft.net/download) and launch Minecraft Java Edition at least once.
2. Download Kable from the [latest releases](../../releases) page or from the [official website](https://kable.kablan.nl/).
3. Install the appropriate package for your operating system:
   - **Windows:** Run the `.msi` installer.
   - **macOS:** Open the `.dmg` file and install the application.
   - **Linux:** Install the available `.deb` package or use another supported distribution format.

**Nightly builds:** I primarily use nightly builds, which may include the latest features and fixes before they reach a stable release. They can also contain regressions or unfinished functionality, so use them at your own risk.

### First Launch

1. Open Kable.
2. Sign in with your Microsoft account.
3. Verify your Minecraft installation directory if necessary.
4. Import an existing profile or create an installation configuration.
5. Browse content, customize your setup, and launch Minecraft.

## ⚠️ Security & Antivirus Notice

Kable may trigger antivirus warnings or Windows SmartScreen prompts, particularly because it is not currently code-signed. Such warnings do not, by themselves, establish whether an application is safe or malicious.

If you encounter a warning:

1. Verify that you downloaded Kable from the official [GitHub Releases](../../releases) page.
2. Review the [source code](../../) and the relevant [GitHub Actions workflows](../../actions) if you want to inspect how builds are produced.
3. Investigate the specific detection before deciding whether to run the application.

**Do not blindly disable antivirus protection or add exclusions simply to bypass a warning.** An unsigned application can be legitimate, but unsigned builds also lack the additional identity verification provided by code signing.

### Privacy & Transparency

Kable is open source and designed to provide transparency into its operation. It does not intentionally collect telemetry or usage analytics for the developer. The application communicates with external services as needed for functionality such as Microsoft authentication, Minecraft metadata, and mod discovery and downloads.

See the [Privacy Policy](PRIVACY.md) for further details.

## 🛠️ Development

Kable is built with:

- **Frontend:** Svelte and TypeScript
- **Desktop framework:** Tauri
- **Backend:** Rust

### Building from Source

#### Prerequisites

- Node.js and npm
- Rust and Cargo
- Platform-specific Tauri build dependencies

#### Setup

```bash
git clone https://github.com/LuckyLuuk12/kable.git
cd kable
npm install
npm run dev
```

#### Production Build

```bash
npm run build
npm run tauri build
```

Refer to the [Tauri documentation](https://v2.tauri.app/start/prerequisites/) for platform-specific build requirements.

## 📚 Documentation

- [Contributing Guidelines](CONTRIBUTING.md)
- [License Information](LICENSE.md)
- [Privacy Policy](PRIVACY.md)
- [Terms of Service](TERMS.md)

## 🤝 Contributing & Support

Contributions, bug reports, and feature suggestions are welcome.

- **Bug reports:** [GitHub Issues](../../issues)
- **Feature requests and discussions:** [GitHub Discussions](../../discussions)
- **Contributing:** [CONTRIBUTING.md](CONTRIBUTING.md)
- **Source code:** [GitHub Repository](../../)

## ⚖️ Legal

### Minecraft Disclaimer

Kable is an unofficial Minecraft launcher and is not affiliated with, endorsed by, or associated with Mojang Studios or Microsoft. Minecraft is a trademark of Mojang Studios.

### License

This project uses a proprietary license. Public source-code availability does not grant permission to copy, modify, or redistribute the software except where expressly permitted by the applicable license. See [LICENSE.md](LICENSE.md) for the full terms.

---

**Made with ❤️ by Luuk Kablan**
