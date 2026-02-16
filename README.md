# Spotlight Genie

<p align="center">
  <img src="./src-tauri/icons/128x128@2x.png" width="128" alt="Spotlight Genie icon" />
</p>

<p align="center">
  A fast, keyboard-first Spotlight/Alfred-style launcher built with <strong>Tauri 2</strong>, <strong>Rust</strong>, and <strong>Vue 3</strong>.
</p>

<p align="center">
  <a href="https://github.com/clarezoe/spotlight-genie/stargazers"><img src="https://img.shields.io/github/stars/clarezoe/spotlight-genie?style=for-the-badge" alt="Stars" /></a>
  <a href="https://github.com/clarezoe/spotlight-genie/network/members"><img src="https://img.shields.io/github/forks/clarezoe/spotlight-genie?style=for-the-badge" alt="Forks" /></a>
  <a href="https://github.com/clarezoe/spotlight-genie/issues"><img src="https://img.shields.io/github/issues/clarezoe/spotlight-genie?style=for-the-badge" alt="Issues" /></a>
  <a href="https://github.com/clarezoe/spotlight-genie/commits/main"><img src="https://img.shields.io/github/last-commit/clarezoe/spotlight-genie?style=for-the-badge" alt="Last Commit" /></a>
</p>

## Quick CTA

- **Run locally now:** `pnpm tauri dev`
- **Build release app:** `pnpm tauri build`
- **Support the project:** [☕ Buy Me a Coffee](https://buymeacoffee.com/clarezoe)

<p>
  <a href="https://buymeacoffee.com/clarezoe"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Support%20Spotlight%20Genie-FFDD00?style=for-the-badge&logo=buymeacoffee&logoColor=000" alt="Buy Me a Coffee" /></a>
</p>

## Why Spotlight Genie

Spotlight Genie is designed for speed and focus:

- Global hotkey launcher (`Cmd+Space` / `Ctrl+Space`)
- App search with real app icon extraction on macOS
- Configurable file search folders + file type icons
- Calculator + currency conversion + web fallback
- Keyboard-first flow (arrows, Enter, Esc, shortcuts)
- Tray integration + hide on blur + persistent window behavior

## Screenshots

> UI screenshot assets can be dropped into `assets/screenshots/` and linked here.
> Current visual preview uses shipped app assets.

<p>
  <img src="./src-tauri/icons/icon.png" width="96" alt="Spotlight Genie preview" />
  <img src="./src-tauri/icons/128x128.png" width="96" alt="Spotlight Genie preview 2" />
  <img src="./src-tauri/icons/128x128@2x.png" width="96" alt="Spotlight Genie preview 3" />
</p>

## Repo Stats

<p>
  <img src="https://github-readme-stats.vercel.app/api?username=clarezoe&repo=spotlight-genie&show_icons=true&rank_icon=github" alt="Repo stats" />
</p>
<p>
  <img src="https://github-readme-stats.vercel.app/api/top-langs/?username=clarezoe&repo=spotlight-genie&layout=compact" alt="Top languages" />
</p>

## Built-in Features

| Area | What you get |
|---|---|
| Launcher | Global hotkey, instant panel toggle, tray menu |
| Search | Fuzzy app search, configurable file search folders |
| Visuals | Native rounded corners, transparent window, icon rendering |
| Productivity | Inline calculator, system commands, web fallback |
| UX | Scrollable results, theme switching (dark/light/auto), settings panel |

## Built-in Plugins

| Plugin | Keyword | Description |
|---|---|---|
| Spotify | `sp` | Play/pause/skip and music-related actions |
| Currency | `cc` | Real-time conversion, natural-language friendly |
| Clipboard | `cb` | Searchable clipboard-style quick actions |

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop Framework | Tauri 2.x |
| Backend | Rust |
| Frontend | Vue 3 + TypeScript + Vite |
| Styling | TailwindCSS 4 |
| Icons | Lucide Vue |
| Package Manager | pnpm |

## Prerequisites

- Rust `1.87+`
- Node.js `20+`
- pnpm `10+`

## Run in Development

```bash
pnpm install
pnpm tauri dev
```

## Build for Release

```bash
pnpm tauri build
```

Artifacts:

- macOS app: `src-tauri/target/release/bundle/macos/Spotlight Genie.app`
- DMG: `src-tauri/target/release/bundle/dmg/Spotlight Genie_0.1.0_aarch64.dmg`

## Project Structure

```text
spotlight-genie/
├─ src/                      # Vue frontend
├─ plugins/                  # TS plugin modules
└─ src-tauri/                # Rust backend + Tauri config
```

## Contributing

1. Fork and clone this repo.
2. Create a branch: `git checkout -b feat/your-feature`
3. Commit: `git commit -m "feat: your feature"`
4. Push and open a PR.

## Support

If Spotlight Genie helps your workflow, support future features:

- [☕ Buy Me a Coffee](https://buymeacoffee.com/clarezoe)
