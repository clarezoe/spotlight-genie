# Spotlight Genie

<p align="center">
  <img src="./src-tauri/icons/128x128@2x.png" width="132" alt="Spotlight Genie Icon" />
</p>

<p align="center">
  <strong>Spotlight speed. Alfred flexibility. Cross-platform simplicity.</strong><br/>
  A keyboard-first launcher for macOS, Windows, and Linux built with Tauri + Rust.
</p>

<p align="center">
  <a href="https://github.com/Kosonens/spotlight-genie/stargazers"><img src="https://img.shields.io/github/stars/Kosonens/spotlight-genie?style=for-the-badge" alt="GitHub stars" /></a>
  <a href="https://github.com/Kosonens/spotlight-genie/network/members"><img src="https://img.shields.io/github/forks/Kosonens/spotlight-genie?style=for-the-badge" alt="GitHub forks" /></a>
  <a href="https://github.com/Kosonens/spotlight-genie/issues"><img src="https://img.shields.io/github/issues/Kosonens/spotlight-genie?style=for-the-badge" alt="GitHub issues" /></a>
  <a href="https://github.com/Kosonens/spotlight-genie/commits/main"><img src="https://img.shields.io/github/last-commit/Kosonens/spotlight-genie?style=for-the-badge" alt="Last commit" /></a>
</p>

## Features

- Global hotkey launcher (`Cmd+Space` / `Ctrl+Space`)
- Fast fuzzy app discovery with native macOS icon extraction
- App search across system apps, user apps, nested `.app` bundles, and Homebrew Cask installs
- Configurable file search folders with file-type icons
- Inline calculator, currency conversion, and web fallback
- Plugin system (`sp`, `cc`, `cb`, `contact`)
- Tray menu controls + settings panel

## Screenshots

![Spotlight Genie Empty Search](./assets/screenshots/search-empty.png)
![Spotlight Genie Query Search](./assets/screenshots/search-query.png)

## Prerequisites

- Node.js `20+`
- pnpm `10+`
- Rust (stable)

## Quick Start

```bash
pnpm install
pnpm tauri dev
```

## Build (Production)

```bash
pnpm tauri build
```

Current macOS outputs:

- `src-tauri/target/release/bundle/macos/Spotlight Genie.app`
- `src-tauri/target/release/bundle/dmg/Spotlight Genie_1.0.0_aarch64.dmg`

## Project Structure

```text
spotlight-genie/
├─ src/             # Vue frontend
├─ plugins/         # TypeScript plugins
├─ src-tauri/       # Rust backend + Tauri config
└─ assets/          # Screenshots and media
```

## Tech Stack

- Tauri 2.x
- Rust
- Vue 3 + TypeScript + Vite
- TailwindCSS 4
- Lucide Vue
- pnpm

## Troubleshooting

- New app not showing after install: search once with the exact app name to trigger lazy index refresh, or restart Spotlight Genie.
- macOS Homebrew apps: ensure the app is installed as a cask (`brew list --cask`) and available in `/Applications` or Homebrew `Caskroom`.

## Contributing

1. Fork the repository.
2. Create a branch: `git checkout -b codex/your-change`.
3. Commit with a clear message.
4. Push and open a PR.

## Support

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Support%20the%20Project-FFDD00?style=for-the-badge&logo=buymeacoffee&logoColor=000)](https://buymeacoffee.com/clarezoe)
