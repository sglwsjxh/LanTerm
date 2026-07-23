# LanTerm

Lightweight LAN web terminal sharing

<p align="center">
  <a href="README-zh.md">中文</a> | <a href="README.md">English</a>
</p>

## Build

### Using Makefile

```bash
make build        # build frontend + Rust release binary
make dev          # build frontend + Rust debug binary
make frontend     # build frontend assets only
make clean        # clean build artifacts
```

### Manual build

```bash
cd frontend && npm install && npm run build
cd .. && cargo build --release
# → target/release/lanterm
```

## Run

```bash
lanterm # default port 8999
lanterm --port 8080 # custom port
lanterm --shell zsh  # custom shell
lanterm --version    # show version
```

## How it works

LanTerm starts an HTTP server on 0.0.0.0:8999, serves a Vue 3 + xterm.js frontend, and upgrades WebSocket connections to a portable-pty shell session. Keyboard input goes through WebSocket binary frames to the PTY writer; PTY output streams back as binary frames to the browser

## Known limitations (v0.1.1)

- No auth — anyone on your LAN can connect
- Single terminal per connection
- Mobile touch selection limited; keyboard via hidden textarea

## Tech stack

- **Backend**: Rust + Axum + tokio + portable-pty + rust-embed
- **Frontend**: Vue 3 + xterm.js + @xterm/addon-fit
- **Protocol**: WebSocket (Binary for PTY I/O, Text JSON for resize control)

# LICENSE

[AGPL-3.0](LICENSE)
