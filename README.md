# AssetForge — Windows Game Asset Organizer

AssetForge v0.1 is a local-first Windows desktop application built with **Tauri 2, React, TypeScript, Rust, and SQLite**.

## Features
- Native folder picker and recursive asset scanning
- Categorization by extension (images, audio, 3D models, fonts, scenes, scripts, video, data, archives)
- SQLite catalog with search, filtering, sorting and favorites
- Image preview for PNG, JPEG, GIF, WebP and BMP up to 8 MB
- Remove catalog entries without deleting original files

## Windows prerequisites
1. Install Node.js LTS.
2. Install Rust with the MSVC toolchain.
3. Install Visual Studio Build Tools with **Desktop development with C++**.
4. Ensure Microsoft Edge WebView2 Runtime is installed.

## Development

```powershell
npm install
npm run tauri dev
```

## Windows installer

```powershell
npm run tauri build
```

Installers are normally written to `src-tauri/target/release/bundle/nsis/` or `msi/`.

## Current limitations
This is initial source code, not a tested or compiled Windows installer. Assets are categorized by extension, not by AI. Files deleted from disk are not automatically removed from the catalog. Rich previews for PSD, Aseprite, SVG, audio and 3D are not implemented. Steam, Epic and itch.io authentication is not yet implemented. Folder scans can be slow for large libraries and inaccessible individual files are skipped.

Original files are never moved, renamed, or deleted by AssetForge.

## Roadmap
- Cached thumbnails, audio playback and 3D previews
- Duplicate detection, tags and collections
- Installed creative software discovery and launching
- Official integrations with itch.io, Epic and Steam where supported
- Rescanning and background indexing
