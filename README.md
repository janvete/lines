# lines

TUI aplikace pro rychlé spouštění často používaných příkazů — ideální pro SSH skripty, aktualizace a cokoliv, co pouštíš pravidelně z terminálu.

## Instalace

```bash
brew install janvete/tools/lines
```

## Použití

```bash
lines              # otevře TUI
lines --dir /cesta/k/datům   # použije jinou datovou složku
```

## Struktura dat

Data jsou běžné `.md` soubory ve složce `~/.lines`. Skupiny = podsložky.

```
~/.lines
├── Synology
│   └── dsmview.md
└── Scripts
    └── update.md
```

### Formát `.md`

Každá sekce začíná nadpisem `# Název příkazu`. Pod ním následují řádky příkazů.

```markdown
# Sysel DSM
dsmview ssh root@192.168.40.15

# Johy DSM
dsmview ssh root@192.168.51.3
```

Sekce bez příkazů znamená "spustit všechny příkazy v tomto souboru".

```markdown
# Spustí vše

# Docker update
ssh root@192.168.53.3 /opt/mealie/update.sh
ssh root@192.168.53.5 /opt/yamtrack/update.sh
```

## Ovládání

| Klávesa | Akce |
|---------|------|
| `↑/↓` nebo `j/k` | pohyb v seznamu |
| `←/→` nebo `h/l` | přepínání panelů |
| `Enter` | spustí vybraný příkaz / všechny příkazy |
| `e` | otevře aktuální `.md` soubor v `$EDITOR` |
| `r` | znovu načte data ze složky |
| `q` nebo `Esc` | ukončí aplikaci |

## Zálohování

Celá datová složka `~/.lines` je obyčejný text — stačí ji zkopírovat na disk, do Gitu nebo iCloudu. Na novém Macu ji přeneseš a `lines` ji rovnou načte.

## Licence

MIT
