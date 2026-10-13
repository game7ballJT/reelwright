# Reelwright app icon

Reelwright's icon is a film reel with five sprocket holes: a flat figure on a rounded tile with a diagonal gradient. It is original artwork
made for Reelwright, drawn in code by `packaging/icons.ps1`; it replaces the upstream project's icon, which
is not used anywhere in this repository.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| Ink | `#1b0f1f` | the tile gradient, top-left |
| Reelwright colour | `#d6264a` | the tile gradient, bottom-right |
| Paper | `#f4efe6` | the figure |
| Spark | `#ffd23f` | the accent |

## Files

- `reelwright.svg`, `reelwright-small.svg`: the vector master (identical; the second name is kept for packaging scripts that expect it), also the hicolor scalable icon.
- `reelwright-1024.png`: 1024 px render on the macOS grid; `reelwright-macos-512.png`: the runtime Dock icon.
- `reelwright.icns`: macOS bundle icon; `reelwright.ico`: Windows icon, 16-256 px.
- `hicolor/<size>/apps/online.jonmcgee.reelwright.png` (16-512) and `hicolor/scalable/...svg`: Linux icon theme.

## Regenerate

`powershell -ExecutionPolicy Bypass -File packaging/icons.ps1` (Windows; uses System.Drawing only).