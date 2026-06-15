# lines

A TUI app for quickly running frequently used commands — ideal for SSH scripts, updates, and anything you run regularly from the terminal.

## Installation

```bash
brew install lines
```

## Usage

```bash
lines              # opens the TUI
lines --dir /path  # use a different data directory
```

## Data structure

Data is stored as plain `.md` files in `~/.lines`. Groups are subdirectories.

```
~/.lines
├── Servers
│   └── web.md
└── Scripts
    └── update.md
```

### `.md` format

Each section starts with `# Command name`. Command lines follow the heading.

```markdown
# Web server
ssh admin@web.example.com

# Database
ssh admin@db.example.com
```

A section without command lines means "run all commands in this file".

```markdown
# Run all

# Docker update
ssh admin@host1.example.com /opt/app/update.sh
ssh admin@host2.example.com /opt/app/update.sh
```

## Configuration

Create `~/.lines/config.toml`:

```toml
terminal = "Ghostty"   # options: Terminal, Ghostty, iTerm
shell = "zsh"          # options: any shell name, default is taken from $SHELL
```

The default terminal is `Terminal` (macOS Terminal.app). The default shell is detected from the `$SHELL` environment variable.

## Controls

| Key | Action |
|-----|--------|
| `↑/↓` or `j/k` | move |
| `←/→` or `h/l` | switch panel |
| `Enter` | run selected command / all commands |
| `e` | open current `.md` file in `$EDITOR` |
| `r` | reload data |
| `q` / `Esc` | quit |

## Backup

The entire `~/.lines` folder is plain text. Copy it to a disk, Git repo, or iCloud. On a new Mac, move it to the same location and `lines` works immediately.

## License

MIT
