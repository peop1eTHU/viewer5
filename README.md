# Viewer5

Viewer5 is a desktop media viewer built with Tauri + SvelteKit.

It scans include folders recursively, supports folder exclusion, and lets you browse images/videos with fast navigation, weighted randomization, and preset management.

## Features

- Recursive media scan (include folders + excluded subfolders)
- Image/video/mixed library modes
- Weighted random order by folder stars (0-5)
- Global flow and current-folder flow navigation
- Presets: save, load, rename, delete
- Viewer, waterfall, and mosaic layouts
- Gallery preloading for smoother browsing
- Keyboard shortcuts

## Tech Stack

- Frontend: Svelte 5 + Tailwind CSS + bits-ui
- Backend: Rust + Tauri v2
- Build: Vite + pnpm

## Getting Started

### Prerequisites

- Node.js (LTS)
- pnpm
- Rust toolchain
- Tauri system dependencies for your OS

### Install

```bash
pnpm install
```

### Run in Development

```bash
pnpm tauri dev
```

### Type/Build Checks

```bash
pnpm check
cd src-tauri && cargo check
```

## Production Build

`pnpm build` only builds frontend assets.

To produce desktop binaries/installers, use:

```bash
pnpm tauri build
```

Bundle outputs are generated in:

```text
src-tauri/target/release/bundle/
```

### Build on macOS

Prerequisites:

- Xcode Command Line Tools (`xcode-select --install`)
- Rust toolchain
- Node.js + pnpm

Build:

```bash
pnpm install
pnpm tauri build
```

Typical outputs:

- `.app`
- `.dmg`

### Build on Windows

Prerequisites:

- Rust toolchain
- Node.js + pnpm
- Visual Studio Build Tools (C++ workload)
- WebView2 Runtime

Build:

```powershell
pnpm install
pnpm tauri build
```

Typical outputs:

- `.msi`
- `.exe` (depending on config)

### Build on Linux

Prerequisites (Ubuntu/Debian example):

```bash
sudo apt update
sudo apt install -y \
	build-essential \
	libwebkit2gtk-4.1-dev \
	libgtk-3-dev \
	libayatana-appindicator3-dev \
	librsvg2-dev \
	patchelf
```

Build:

```bash
pnpm install
pnpm tauri build
```

Typical outputs:

- `.AppImage`
- `.deb`
- `.rpm` (depending on host tools)

## Cross-Platform Release via GitHub Actions

If you only have macOS, use CI to build Windows/macOS/Linux artifacts automatically.

This repo includes workflow:

- `.github/workflows/release.yml`

It will:

- build on `macos-latest`, `windows-latest`, and `ubuntu-22.04`
- upload build artifacts
- create/update a GitHub Release and attach generated bundles

### Trigger automatic release by tag

```bash
git tag v0.1.0
git push origin v0.1.0
```

When tag `v*` is pushed, workflow runs and publishes release assets automatically.

### Manual release trigger

You can also run the workflow from GitHub Actions UI with `workflow_dispatch` and provide a tag (for example `v0.1.1`).

### Required GitHub settings

- Repository Actions permission should allow workflow to write `contents`.
- Default `GITHUB_TOKEN` is used by workflow to publish release.

### Optional code signing

- macOS signing/notarization and Windows code signing are optional for internal/testing builds.
- For public distribution, configure signing secrets and Tauri signing environment variables.

## Usage

1. Add one or more include folders.
2. Optionally add excluded folders.
3. Choose media mode (`images`, `videos`, or `mixed`).
4. Click `Shuffle`/navigate with `Next`/`Previous`.
5. Use preset input + `Save` to persist your setup.

## Project Structure

- `src/routes/+page.svelte`: main app UI
- `src-tauri/src/lib.rs`: backend state, scan, navigation, presets, and Tauri commands
- `src-tauri/tauri.conf.json`: Tauri app configuration

## License

This project is licensed under the MIT License. See `LICENSE` for details.
