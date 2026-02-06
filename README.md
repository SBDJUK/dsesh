# dsesh

**A Sesh-compatible, non-tmux terminal session manager**

`dsesh` is a small, terminal-agnostic session launcher inspired by [Sesh](https://github.com/joshmedeski/sesh).  

I liked the idea of preconfigured sessions with TOML files, but I didn’t always want to use tmux.  
`dsesh` lets you **use your existing Sesh TOML configurations** without creating tmux sessions. It runs commands in your **current terminal**.

## Features

- Fully compatible with Sesh TOML session files
- Recursive imports supported
- Launches commands directly in the current terminal (no tmux)
- Works with fzf for interactive session selection
- Simple CLI with two commands: `list` and `connect`
- `list` command supports an optional filter

## Building

Requires Rust and Cargo:

```bash
git clone <repo-url>
cd dsesh
cargo build --release
```

## Installation

```bash
cargo install --path .
```

## Usage

I personally use

```bash
dsesh connect "$(dsesh list $1 | fzf)"
```

or

```bash
dsesh connect "Session Name"
```

I use a short alias to make the fzf version easier to use.
