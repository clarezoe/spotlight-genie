# TODO

## Completed
- [x] Project scaffold (Tauri 2 + Vue 3 + TailwindCSS 4)
- [x] Frameless transparent window with glassmorphism panel
- [x] Global hotkey (Cmd+Space / Ctrl+Space)
- [x] System tray with Show/Quit menu
- [x] App indexing (macOS, Windows, Linux)
- [x] Fuzzy search with ranking
- [x] Inline calculator (meval)
- [x] Web search fallback
- [x] System commands (sleep, lock)
- [x] Keyboard navigation (arrows, Enter, Esc, Cmd+1-8)
- [x] Plugin system with GeniePlugin interface
- [x] Spotify plugin (sp keyword)
- [x] Currency plugin (cc keyword)
- [x] Clipboard history plugin (cb keyword)
- [x] SearchBar, ResultList, ResultItem, ActionBar, CategoryBadge components
- [x] Dark glassmorphism UI (Outfit + Work Sans fonts)
- [x] Rust warning cleanup (deprecated macOS private API path and dead-code warnings)
- [x] Non-blocking file search index cache to prevent search hangs
- [x] macOS launcher polish (Dock hidden + app icons resolved in search results)
- [x] Contacts integration now reads real system contacts and matches in normal search
- [x] Show launcher window immediately on app startup (without hotkey trigger)
- [x] Search ranking tuned so exact app matches outrank unrelated system commands
- [x] Search stabilization: ignore stale async responses and re-rank exact title matches first
- [x] macOS workspace behavior: launcher window appears across all desktops/spaces
- [x] Robust app ranking + hotkey focus toggle fix for cross-desktop invocation
- [x] Reworked app matching (normalized exact/prefix/acronym) and app activation on show
- [x] Settings menu item in tray menu for quick access
- [x] Plugin management in settings (enable/disable plugins)
- [x] Improved app launching with macOS 'open -a' command for better activation
- [x] Enhanced app scanning to find nested apps (Edge, Chrome in subdirectories)
- [x] macOS app discovery now includes Homebrew Cask installations
- [x] Search auto-refreshes stale app index on app-miss queries (cooldown-protected)
- [x] Contacts plugin with error handling and extended timeout

## Next
- [ ] File search plugin (search filenames in common directories)
- [ ] Frecency scoring (track usage, boost frequently used items)
- [ ] Auto-start on login
- [ ] Window appear/dismiss animations
- [ ] Result character highlighting (matched fuzzy chars)
- [ ] Spotify OAuth2 Web API integration
- [ ] User-installable plugins from directory
- [ ] WASM plugin sandboxing
- [ ] Custom themes
