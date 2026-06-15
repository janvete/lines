# lines

A TUI app for quickly running frequently used commands — ideal for SSH scripts, updates, and anything you run regularly from the terminal.

## Installation

```bash
brew tap janvete/tools
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
| `Enter` | run selected command / all commands in the current terminal |
| `o` | open selected command / all commands in a new terminal window |
| `c` | open custom command mode for the current file |
| `e` | open current `.md` file in `$EDITOR` |
| `r` | reload data |
| `q` / `Esc` | quit |

## Custom command mode

Press `c` on a selected file to run a custom command against specific lines from that file.

For a file containing:

```markdown
# PVE1
ssh root@host1.example.com

# PVE2
ssh root@host2.example.com

# PVE3
ssh root@host3.example.com
```

Press `c`, select lines with `Space`, type `lsblk`, and press `Enter`. `lines` will run:

```bash
ssh root@host2.example.com lsblk
ssh root@host3.example.com lsblk
```

Use `{}` as a placeholder if your lines are just addresses:

```bash
ssh {} lsblk
```

## History

Every command run through `lines` is appended to `~/.lines/history.log`:

```
2026-06-15 08:30:15 [Servers/web.md] Main server: ssh admin@web.example.com
2026-06-15 08:31:02 [Scripts/update.md] Run all: ssh admin@host1.example.com /opt/app/update.sh; ssh admin@host2.example.com /opt/app/update.sh
```

## Backup

The entire `~/.lines` folder is plain text. Copy it to a disk, Git repo, or iCloud. On a new Mac, move it to the same location and `lines` works immediately.

## License

MIT
