# ViralMind Desktop

<p align="center" width="100%">
    <img src="https://github.com/user-attachments/assets/304e2bd6-9584-4d4b-afdb-71d759d91846">
</p>

A desktop application for contributing to the world's largest dataset for multimodal computer-use agents. Earn $VIRAL tokens in two ways: record your desktop interactions to train better computer-use AI, or provide secure virtual desktop infrastructure for deploying these agents. 

Built from the ground up with privacy and security as core principles. See our [Privacy Policy](PRIVACY.md) for details.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Requirements

### Windows

1. Install bun
```bash
powershell -c "irm bun.sh/install.ps1 | iex"
```
2. Install Microsoft C++ Build Tools
3. Install Rust
```bash
winget install --id Rustlang.Rustup
```

--- 


### Debian Linux / Ubuntu WSL

1. Install bun
```bash
curl -fsSL https://bun.sh/install | bash
```

2. Install Tauri pre-requisites
```
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Development

```bash
bun install
```

```bash
bun tauri dev
```

```bash
bun tauri build
```
