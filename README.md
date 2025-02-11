# ViralMind Desktop

> 🚧 Work in progress - coming soon

<p align="center" width="100%">
    <img src="https://github.com/user-attachments/assets/304e2bd6-9584-4d4b-afdb-71d759d91846">
</p>

A desktop application for contributing to the world's largest dataset for multimodal computer-use agents. Earn $VIRAL tokens in two ways: record your desktop interactions to train better computer-use AI, or provide secure virtual desktop infrastructure for deploying these agents. 

Built from the ground up with privacy and security as core principles. See our [Privacy Policy](PRIVACY.md) for details.

## Recording System

Our recording system is designed specifically for training powerful computer-use AI agents through data-driven approaches. Key features include:

### Quest System

<p align="center" width="100%">
    <img src="https://github.com/user-attachments/assets/8f516e95-1a5d-49a0-9c27-aa932b7cd6d5">
</p>

- AI-generated quests, generated from random UI elements from thousands available on your screen

- Ensures diverse, instruction-following demonstration trajectories

- Structured with subobjectives to help AI models break down long trajectories into step-by-step plans

- Example quest shown above: creating a spreadsheet with specific requirements and subobjectives

### Recording Format

Recordings are two files and stored in `%LOCALAPPDATA%\ai.viralmind.desktop\recordings\` on Windows and `${HOME}/Library/Application Support/ai.viralmind.desktop/recordings/` on MacOS:

- .mp4 video capture

- .jsonl event log capturing detailed interaction data

Sample event log format:

```json
{"event":"quest_started","data":{"id":"spreadsheet_creation_01","title":"Create a New Spreadsheet","description":"Open Excel or Google Sheets and create a new spreadsheet with at least 3 columns and 5 rows of data","reward":10},"time":1738564880000}
{"data":{"output":"ffmpeg version 7.1-essentials_build-www.gyan.dev Copyright (c) 2000-2024 the FFmpeg developers"},"event":"ffmpeg_stderr","time":1738564880824}
{"data":{"x":1303.0,"y":1347.0},"event":"mousemove","time":1738564880935}
{"data":{"x":1303.0,"y":1347.0},"event":"mousemove","time":1738564880935}
{"data":{"button":"Left"},"event":"mousedown","time":1738564883325}
{"event":"subobjective_completed","data":{"quest_id":"spreadsheet_creation_01","objective":"Open spreadsheet application","index":1},"time":1738564883525}
{"data":{"key":"H"},"event":"keydown","time":1738564891760}
{"event":"subobjective_completed","data":{"quest_id":"spreadsheet_creation_01","objective":"Create new document","index":2},"time":1738564892000}
{"event":"quest_completed","data":{"id":"spreadsheet_creation_01","reward_earned":10,"time_taken":12000},"time":1738564892500}
```

# Development

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

### MacOS

1. Install bun
```bash
curl -fsSL https://bun.sh/install | bash
```

2. Install Build Tools
```bash
xcode-select --install
```

3. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```


### Debian Linux / Ubuntu WSL

1. Install bun
```bash
curl -fsSL https://bun.sh/install | bash
```

2. Install Tauri pre-requisites
```bash
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
