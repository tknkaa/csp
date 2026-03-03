# csp — Copilot Session Picker

> A fast terminal UI for browsing and resuming GitHub Copilot CLI sessions.

<video src="demo.mp4" controls autoplay loop muted width="800"></video>

---

## Features

- **Browse sessions** — paginated list sorted by most recently active
- **At-a-glance info** — time, message count, and a preview of your first message
- **Detail panel** — full session metadata including working directory and session ID
- **One-keystroke resume** — press `Enter` to `cd` into the session's directory and relaunch `copilot --resume`

## Installation

```sh
cargo install --git https://github.com/tknkaa/csp
```

The `csp` binary will be available in your `$PATH` via `~/.cargo/bin`.

## Usage

```sh
csp
```

csp reads sessions from `~/.copilot/session-state/` automatically.

### Key bindings

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `←` / `h` | Previous page |
| `→` / `l` | Next page |
| `Enter` | Resume selected session |
| `q` / `Esc` | Quit |

## License

MIT
