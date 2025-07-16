# Kable
### An Unofficial Minecraft Launcher

Kable is a modern, feature-rich Minecraft launcher built with Tauri and Svelte. It provides a sleek alternative to the official Minecraft launcher with advanced mod management, installation customization, and seamless authentication.

![License](https://img.shields.io/badge/license-Proprietary-red.svg)
![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)

## 🚀 Features

### ✨ Current Features
- **Multi-Account Management**: Support for Microsoft/Mojang accounts with automatic token refresh
- **Mod Loader Support**: Native support for Fabric, Forge, Quilt, and NeoForge
- **Installation Management**: Create and manage multiple Minecraft installations with different configurations
- **Advanced Mod Management**: Per-installation mod directories with automatic linking
- **Memory Allocation**: Custom JVM memory settings per installation
- **Version Detection**: Automatic detection of mod loaders and versions
- **Quick Launch**: Launch recent installations with one click
- **Comprehensive Logging**: Detailed launch logs and process monitoring
- **Clean UI**: Modern, intuitive interface built with Svelte

### 🎯 Mod Loader Detection
Kable automatically detects and supports:
- **Vanilla Minecraft** - Standard Minecraft installations
- **Fabric** - Lightweight modding platform
- **Forge** - Traditional modding framework
- **Quilt** - Fork of Fabric with additional features
- **NeoForge** - Modern fork of Minecraft Forge
- **OptiFine** - Performance and visual enhancement mod
- **Iris** - Shader mod for Fabric

### 🔧 Advanced Features
- **Custom Game Directories**: Separate game directories per installation
- **JVM Arguments**: Custom Java arguments and memory allocation
- **Native Libraries**: Automatic extraction and management of native dependencies
- **Profile Management**: Import and manage existing Minecraft launcher profiles
- **Cross-Platform**: Works on Windows, macOS, and Linux

## 🎮 Getting Started

### Prerequisites
- Java 8 or higher installed on your system
- A valid Minecraft account (Java Edition)
- Windows 10+, macOS 10.15+, or a modern Linux distribution

### Installation
1. Download the latest release from the [Releases](../../releases) page
2. Install the application for your platform:
   - **Windows**: Run the `.msi` installer
   - **macOS**: Open the `.dmg` file and drag Kable to Applications
   - **Linux**: Install the `.deb` package or extract the `.tar.gz` archive

### First Launch
1. Open Kable
2. Sign in with your Microsoft account
3. Set your Minecraft installation directory (if different from default)
4. Browse or create installations
5. Launch and enjoy!

## 🛠️ Development

### Building from Source

#### Prerequisites
- Node.js 18+ and npm
- Rust (latest stable version)
- Platform-specific build tools

#### Setup
```bash
# Clone the repository
git clone https://github.com/LuckyLuuk12/kable.git
cd kable

# Install dependencies
npm install

# Start development server
npm run dev
```

#### Building for Production
```bash
# Build the application
npm run build

# Build Tauri app
npm run tauri build
```

## 🌟 Future Goals

### Short-term (v0.2-0.3)
- [ ] **Mod Marketplace Integration**: Browse and install mods directly from CurseForge and Modrinth
- [ ] **Shader Pack Management**: Built-in shader pack installation and management
- [ ] **Resource Pack Manager**: Easy resource pack installation and switching
- [ ] **Instance Export/Import**: Share configurations with friends
- [ ] **Auto-Updater**: Automatic application updates

### Medium-term (v0.4-0.6)
- [ ] **Mod Dependency Resolution**: Automatic mod dependency management
- [ ] **Version Installer**: Install any Minecraft version directly from the launcher
- [ ] **Backup System**: Automatic world and configuration backups
- [ ] **Performance Monitoring**: Built-in FPS and performance tracking
- [ ] **Server Integration**: Quick connect to favorite servers

### Long-term (v0.7+)
- [ ] **Plugin System**: Extensible architecture for community plugins
- [ ] **Cloud Sync**: Synchronize configurations across devices
- [ ] **Mod Development Tools**: Integrated development environment for mod creators
- [ ] **Advanced Analytics**: Detailed performance and usage analytics
- [ ] **Community Features**: Share configurations and discover new content

## 📚 Documentation

- [Contributing Guidelines](CONTRIBUTING.md)
- [License Information](LICENSE.md)
- [Privacy Policy](PRIVACY.md)
- [Terms of Service](TERMS.md)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details on how to:
- Report bugs and request features
- Submit pull requests
- Set up your development environment
- Follow our code style and conventions

## ⚖️ Legal

### Important Notice
Kable is an **unofficial** Minecraft launcher and is not affiliated with or endorsed by Mojang Studios or Microsoft. Minecraft is a trademark of Mojang Studios.

### License
This project is proprietary software. While the source code is publicly available for transparency and community contributions, **no license is granted to copy, modify, or distribute this software**. See [LICENSE.md](LICENSE.md) for full details.

### Privacy
Kable respects your privacy. We do not collect, store, or transmit any personal data. All authentication is handled directly between your device and Microsoft/Mojang servers. See our [Privacy Policy](PRIVACY.md) for details.

## 📞 Support

- **Bug Reports**: [GitHub Issues](../../issues)
- **Feature Requests**: [GitHub Discussions](../../discussions)
- **Documentation**: Check the docs folder or GitHub Wiki

## 🙏 Acknowledgments

- **Mojang Studios** - For creating Minecraft
- **Tauri Team** - For the excellent desktop app framework
- **Svelte Team** - For the reactive UI framework
- **Rust Community** - For the amazing ecosystem
- **Mod Loader Teams** - Fabric, Forge, Quilt, and NeoForge developers

---

**Made with ❤️ by Luuk Kablan**

*Kable is not affiliated with Mojang Studios, Microsoft, or any mod loader development teams.*
