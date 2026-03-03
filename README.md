# csp — Copilot Session Picker

> A fast terminal UI for browsing and resuming GitHub Copilot CLI sessions.

<!-- screenshot goes here -->

---

## Features

- **Browse sessions** — paginated list sorted by most recently active
- **At-a-glance info** — time, message count, and a preview of your first message
- **Detail panel** — full session metadata including working directory and session ID
- **One-keystroke resume** — press `Enter` to `cd` into the session's directory and relaunch `copilot --resume`

## Installation

```sh
cargo install --path .
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

## Build from source

```sh
git clone https://github.com/yourname/csp
cd csp
cargo build --release
./target/release/csp
```

## License

MIT
