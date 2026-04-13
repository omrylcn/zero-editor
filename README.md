# Pi Editor

A lightweight, web-based code editor built with Rust for resource-constrained devices like the Raspberry Pi Zero.

**3 MB binary. ~2-3 MB RAM. Zero dependencies.**

## Why?

VS Code Remote SSH is too heavy for a Pi Zero (416 MB RAM). JupyterLab is too heavy. Nano works but lacks a modern editing experience. Pi Editor fills this gap: a real code editor with syntax highlighting, file management, and an integrated terminal — all served from a single binary that barely touches your RAM.

## Features

- **Code Editor** — CodeMirror-powered editor with syntax highlighting for Python, Rust, JavaScript, Shell, TOML, YAML, Markdown, HTML, CSS, and more
- **File Explorer** — Browse, open, and navigate your project directory
- **Integrated Terminal** — Run commands directly from the browser with command history (Up/Down arrows) and tab completion
- **Run Button** — One-click execution with automatic language detection:
  - `.py` → `uv run python`
  - `.rs` → `rustc` + run
  - `.js` → `node`
  - `.sh` → `bash`
- **File Transfer** — Drag-and-drop upload from your desktop + download button
- **Keyboard Shortcuts** — `Ctrl+S` save, `Ctrl+Enter` run
- **Dark Theme** — Dracula theme, easy on the eyes

## Quick Start

### Build from source

```bash
# Clone
git clone https://github.com/omrylcn/pi-editor.git
cd pi-editor

# Build
cargo build --release

# Run (edit WORKSPACE path in src/main.rs first)
./target/release/pi-editor
```

Open `http://localhost:3000` in your browser.

### Deploy to Raspberry Pi

```bash
# Cross-compile for aarch64
cross build --release --target aarch64-unknown-linux-gnu

# Copy binary to Pi
scp target/aarch64-unknown-linux-gnu/release/pi-editor user@pi-address:~/

# SSH and run
ssh user@pi-address
./pi-editor
```

Then access from your desktop browser: `http://<pi-ip>:3000`

### SSH Port Forward (optional)

```bash
ssh -L 3000:localhost:3000 user@pi-address
# Then open http://localhost:3000
```

## Configuration

Edit the `WORKSPACE` constant in `src/main.rs` to point to your project directory:

```rust
const WORKSPACE: &str = "/home/your-user/your-project";
```

## Tech Stack

- **Backend:** Rust + Actix Web (~3 MB binary, stripped + LTO)
- **Frontend:** CodeMirror 5 (loaded from CDN, zero storage on device)
- **Dependencies:** None at runtime — single static binary

## Resource Usage

| | Pi Editor | VS Code Remote | JupyterLab |
|---|---|---|---|
| RAM | ~2-3 MB | ~200+ MB | ~100+ MB |
| Disk | 3 MB | 200+ MB | 50+ MB |
| Startup | instant | 10-30s | 5-15s |

## License

MIT
