# Ski Jump 3

An SDL3-based Rust port of Ski Jump 3 (SJ3). Windows release archives ship a
single folder: `skijump3.exe` with its data files flat next to it — no
subfolders, no installer. Sprites and the main background are baked into the
exe; everything else beside it is editable and customizes the game. Debug
builds read `game/assets/` from the workspace instead.

## Requirements

- Rust 1.88 or newer
- CMake and a C compiler (SDL3 is built from source and linked statically)
- Any desktop OS SDL3 supports (Linux needs X11 or Wayland plus audio
  development dependencies); release zips are cut for Windows x86_64 only

On Debian or Ubuntu, the packages used by CI can be installed with:

```sh
sudo apt-get install cmake ninja-build pkg-config libasound2-dev libpulse-dev \
  libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxfixes-dev libxi-dev \
  libxss-dev libxtst-dev libwayland-dev libxkbcommon-dev libdrm-dev libgbm-dev \
  libgl1-mesa-dev libegl1-mesa-dev libdbus-1-dev libudev-dev
```

## Build And Run

```sh
cargo build --release --locked
cargo run --release --locked
```

Use `cargo run --locked -- --help` for startup options. If hardware rendering
does not work, start with `--sw-rendering`.

## Controls

- Arrow keys move through menus; Enter selects and Escape goes back.
- During a jump, Right starts the in-run and leans forward, Up takes off, and
  Left corrects or leans back.
- `T` starts a telemark landing; `R` starts a two-footed landing.
- `P` pauses during flight or landing, `S` saves a replay from the result
  screen, and `F5` resets wind where the game mode permits it.
- `+` and `-` adjust the start gate where the game mode permits it.

The five main jump keys can be changed in Setup.

## Save Data

The game is portable: saves live next to the executable, not in the OS
profile. On first start (no language saved yet) the game opens with the
WELCOME language picker, then plays the intro replay.

The TOML save formats are versioned and strict: a `config.toml`,
`players.toml`, or `hiscores.toml` with an unsupported format version or
invalid fields is a startup error naming the file — fix the file or re-run
with `-c`, `-r`, or `-z`. Missing files are created from bundled defaults
(records and players from the shipped data, config from built-in defaults).
Invalid cup saves are ignored.

The startup reset flags overwrite their target immediately:

- `-c` replaces `config.toml` with default settings.
- `-r` replaces `hiscores.toml` with bundled records.
- `-z` replaces `hiscores.toml` with zeroed records.

Only one reset flag may be supplied. Unknown and conflicting command-line
options are errors.

## Licensing

The Rust source code is available under the GNU GPL version 3 only; see
[LICENSE](LICENSE). The bundled SJ3 game assets originate from the original
Ski Jump 3 by Ville Kononen and are redistributed with the author's permission.
