# LanTerm

Lightweight LAN web terminal sharing. Run it, open your phone browser, get a terminal.

## Build

```bash
cd frontend && npm install && npm run build
cd .. && cargo build --release # → target/release/lanterm (single binary, frontend embedded)
```

## Run

```bash
lanterm # default port 8999
lanterm --port 8080 # custom port
lanterm --shell zsh # custom shell
```

## How it works

LanTerm starts an HTTP server on 0.0.0.0:8999, serves a Vue 3 + xterm.js frontend, and upgrades WebSocket connections to a portable-pty shell session. Keyboard input goes through WebSocket binary frames to the PTY writer; PTY output streams back as binary frames to the browser.

## Known limitations (v0.1)

- No auth — anyone on your LAN can connect
- Single terminal per connection
- LAN IP detection may pick wrong NIC on multi-NIC machines
- Mobile touch selection limited; keyboard via hidden textarea

## Tech stack

- **Backend**: Rust + Axum + tokio + portable-pty + rust-embed
- **Frontend**: Vue 3 + xterm.js + @xterm/addon-fit
- **Protocol**: WebSocket (Binary for PTY I/O, Text JSON for resize control)
