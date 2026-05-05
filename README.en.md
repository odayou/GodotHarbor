# Godot Harbor

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT) [![GitHub Stars](https://img.shields.io/github/stars/odayou/GodotHarbor?style=social)](https://github.com/odayou/GodotHarbor) [![GitHub Downloads](https://img.shields.io/github/downloads/odayou/GodotHarbor/total?color=brightgreen)](https://github.com/odayou/GodotHarbor/releases) [![Gitee Downloads](https://img.shields.io/badge/Gitee-China_CDN-orange)](https://gitee.com/odayou/godot-harbor/releases)

中文 | **[English](README.md)**

Godot Harbor is a standalone desktop application that provides Godot developers with a unified plugin repository, project binding management, and engine download management.

## Screenshots

<table>
  <tr>
    <td align="center"><b>Onboarding</b></td>
    <td align="center"><b>Quick Actions</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-16.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-17.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>Projects</b></td>
    <td align="center"><b>Asset Library</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-02.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-08.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>Engines</b></td>
    <td align="center"><b>Install / Upgrade Engine</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-09.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-12.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>Plugin Import</b></td>
    <td align="center"><b>Dark Theme</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-04.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/theme_black.jpeg" width="480"/></td>
  </tr>
</table>

<details>
<summary>📸 More Screenshots</summary>

<table>
  <tr>
    <td align="center"><b>Dashboard</b></td>
    <td align="center"><b>Command Palette</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-00.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-01.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>Plugin Binding</b></td>
    <td align="center"><b>Project Details</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-06.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-03.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>Backup & Restore</b></td>
    <td align="center"><b>Quick Bind</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-15.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-05.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>Binding Graph View</b></td>
    <td align="center"><b>Launch Engine</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-07.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-10.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>Settings</b></td>
    <td align="center"><b>Storage Info</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-14.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-18.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>About</b></td>
    <td align="center"><b>Credits</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-19.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-20.jpeg" width="480"/></td>
  </tr>
</table>

</details>

## Features

- **Plugin Management**: Import plugins from local directories or Git repositories, manage plugin versions
- **Project Management**: Auto-scan or manually add Godot projects
- **Plugin Binding**: Select plugins and versions for projects
- **One-Click Apply**: Mount plugins to project addons directories
- **Engine Management**: Discover, download, and register Godot engines

## Tech Stack

- **Desktop Framework**: Tauri 2.x
- **Backend**: Rust
- **Frontend**: Vue 3 + TypeScript + TailwindCSS
- **Data Persistence**: Local JSON files

## Requirements

- **Node.js**: 18.0 or higher
- **Rust**: 1.86.0 or higher (GNU toolchain recommended)
- **OS**: Windows 10/11, macOS 10.15+, or Linux

## 💡 Windows Tips

### Dev Environment Optimization

During development, Rust, Node.js, and other tools download many dependencies that can consume significant C: drive space. Here are optimization tips:

#### 1. Rust Environment Setup (Recommended)

Use the **GNU toolchain** and install Rust on D: drive:

```powershell
# Set environment variables (add to system environment variables permanently)
$env:CARGO_HOME = "D:\Rust\.cargo"
$env:RUSTUP_HOME = "D:\Rust\.rustup"

# Install GNU toolchain
rustup toolchain install 1.86.0-x86_64-pc-windows-gnu
rustup default 1.86.0-x86_64-pc-windows-gnu

# Verify
rustc --version
rustup show home  # Should show D:\Rust\.rustup
```

#### 2. Node.js Global Packages (Optional)

To move npm global packages to D: drive:

```powershell
# Set npm global install path
npm config set prefix "D:\NodeJS\npm"

# Verify
npm config get prefix
```

#### 3. App Data Storage

The app uses Tauri's standard **cross-platform app data directory** for runtime data (project info, plugin config, etc.):

- **Windows**: `%APPDATA%/godot-harbor`
- **macOS**: `~/Library/Application Support/godot-harbor`
- **Linux**: `~/.config/godot-harbor`

## Quick Start

### Option 1: Web Preview (Frontend Only)

If you only want to preview the frontend:

```bash
# Install dependencies
npm install

# Start frontend dev server
npm run dev
```

Then visit `http://localhost:1420/` in your browser.

**Note**: The web version cannot use backend features (file system operations, plugin imports, etc.). It is for frontend preview only.

### Option 2: Desktop App (Full Features)

#### All Platforms (Windows/macOS/Linux)

```bash
# 1. Install dependencies
npm install

# 2. Start Tauri dev mode
npm run tauri dev
```

The first run compiles the Rust backend, which may take 10-20 minutes.

## Production Build

### Local Build

```bash
# Build desktop app
npm run tauri build
```

After building, the installer is located in `src-tauri/target/release/bundle/`.

### One-Command Release

```bash
# Patch version (e.g. v0.1.4 → v0.1.5)
npm run release -- patch

# Minor version (e.g. v0.1.4 → v0.2.0)
npm run release -- minor

# Major version (e.g. v0.1.4 → v1.0.0)
npm run release -- major

# Explicit version
npm run release -- 0.2.0

# Custom commit message (\n for newlines)
npm run release -- patch -m "feat: new feature\nfix: bug fix"
```

This command automatically: bumps version → git commit → create tag → push → CI builds and releases.

> **Note**: `--` separates npm args from script args and cannot be omitted.

### GitHub Actions Build (Recommended)

The project has GitHub Actions workflow configured for **multi-platform parallel builds**.

#### Trigger

The workflow is **manual only** and does not auto-build on push:

1. Go to the **Actions** page on GitHub
2. Select **Build Godot Harbor** workflow
3. Click **Run workflow**
4. Enter version number (e.g. `v1.0.0`) and optional build reason
5. Click **Run workflow** to start

#### Build Targets

| Platform | Architecture | Output Format |
|----------|-------------|---------------|
| Windows | x86_64 | NSIS (.exe) |
| macOS | Universal (ARM + Intel) | DMG |
| Linux | x86_64 | DEB |

#### Artifacts

After building, all platform installers are automatically packaged into a GitHub Release (Draft state), available from the **Releases** page.

## Project Structure

```
godot-harbor/
├── src/                    # Vue frontend
│   ├── api/               # API wrappers
│   ├── components/         # Vue components
│   │   └── layout/        # Layout components
│   ├── composables/        # Composables
│   ├── router/            # Router config
│   ├── stores/            # Pinia stores
│   ├── types/             # TypeScript types
│   ├── views/             # Page views
│   ├── App.vue            # Root component
│   ├── main.ts            # Entry point
│   └── style.css          # Global styles
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── commands/      # Tauri commands
│   │   ├── hot_update/    # Hot update module
│   │   ├── linker/        # Binding management
│   │   ├── models/        # Data models
│   │   ├── plugin_manager/# Plugin management
│   │   ├── scanner/       # Project scanner
│   │   ├── storage/       # Storage module
│   │   ├── update_scheduler/ # Update scheduler
│   │   ├── lib.rs         # Library entry
│   │   └── main.rs        # Main program
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri config
├── workers/               # Cloudflare Worker (update endpoint)
├── scripts/               # Build scripts (manifest generation, etc.)
├── .github/workflows/     # CI/CD workflows
├── docs/                  # Documentation
└── package.json           # Node.js dependencies
```

## Documentation Index

| Document | Description |
|----------|-------------|
| [Update System](docs/update-system.md) | Update strategy, architecture, release flow (full/hot update), Worker config |
| [UI Style Guide](docs/design/UI风格指南.md) | UI design specs and theme schemes |
| [Plugin Management Analysis](docs/technical/插件管理分析.md) | Plugin system technical architecture |
| [Engine Auto-Discovery](docs/technical/引擎自动发现.md) | Godot engine identification and discovery |
| [Product Plan v0.1](docs/planning/产品规划_v0.1.md) | Initial product planning |
| [Iteration Plan v0.2](docs/planning/迭代计划_v0.2.md) | v0.2 iteration plan |

## Development Guide

### Frontend Development

```bash
# Start frontend dev server only
npm run dev

# Build frontend assets
npm run build

# Type check
npm run typecheck
```

### Backend Development

```bash
# Check Rust code
cd src-tauri
cargo check

# Run tests
cargo test

# Build release
cargo build --release
```

### Full Development Workflow

```bash
# 1. Install dependencies
npm install

# 2. Start dev server (frontend + backend)
npm run tauri dev

# 3. Test in browser or desktop app

# 4. Build production version
npm run tauri build
```

### Custom Icon Replacement

```bash
# Replace all icons
python replace-icons.py logo.png ./src-tauri/icons

# Custom exclude prefixes
python replace-icons.py logo.png ./src-tauri/icons --exclude tray_,debug_
```

## FAQ

### 1. Compilation takes too long

Rust first-time compilation downloads and compiles many dependencies. This is normal. Subsequent builds are much faster.

### 2. Symlink permission denied on Windows

Creating symlinks on Windows requires admin privileges. The app automatically falls back to junction or copy mode.

### 3. Git clone failed

Make sure Git is installed and network is working. Some regions may need proxy configuration.

### 4. Frontend hot reload not working

Try clearing browser cache or using incognito mode.

### 5. Backend API calls fail

Make sure you're running in desktop app mode (`npm run tauri dev`), not frontend-only mode (`npm run dev`).

### 6. Project version number incomplete

For older projects, re-scan or remove and re-add to get the full Godot version number.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Create a Pull Request

## License

MIT License

## Contact

- Project: [GitHub](https://github.com/odayou/GodotHarbor) | [Gitee (China Mirror)](https://gitee.com/odayou/godot-harbor)
- Issues: [GitHub Issues](https://github.com/odayou/GodotHarbor/issues) | [Gitee Issues](https://gitee.com/odayou/godot-harbor/issues)
- Downloads: [GitHub Releases](https://github.com/odayou/GodotHarbor/releases) | [Gitee Releases (China CDN)](https://gitee.com/odayou/godot-harbor/releases)
- Email: gbytl@sina.cn

---

**Note**: This project uses Rust 1.86.0 GNU toolchain compiled and run natively on Windows, no WSL required.
