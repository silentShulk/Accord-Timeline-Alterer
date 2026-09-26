# Accord's Timeline Alterer (ATA)

ATA is a mod manager for NieR:Automata for both Linux and Windows.

This repository is ATA's backend. It is both:

- a **CLI** (`ATA`), which GUIs written with any technology can call
- a **Rust library** (`ata`), which Rust GUIs (ShellUI) and the Launcher use directly

## Features

* Install mods from `.zip`, `.7z`, `.rar` archives, or from an already extracted folder
* Uninstall mods, restoring the original game files they replaced
* List installed mods, sorted by type, date, status, name or size
* Enable/Disable mods without uninstalling them
* Detect file conflicts between mods, and optionally overwrite them
* Manage settings/preferences
* Find the game in the Steam libraries, and launch it

Supported mods: textures, player/weapon/world models, cutscene replacements, ReShade presets
(see [docs/RESHADE.md](docs/RESHADE.md)) and DLL mods.

### Safety

- Every operation is all-or-nothing: if something fails halfway, the game folder and the
  data file are left exactly as they were.
- Original game files replaced by a mod (e.g. cutscenes) are kept in `<game>/.ata-backup/`
  and restored when the mod is disabled or uninstalled.
- Archive entries trying to escape the extraction folder are ignored, and ATA never deletes a
  folder it didn't create.

## Installation/Usage

### Installation

1. Copy the repo `git clone https://github.com/silentShulk/Accord-Timeline-Alterer.git`
2. cd into the repo `cd Accord-Timeline-Alterer`
3. Build the project `cargo build --release`
4. Run the executable `./target/release/ATA`

The ATA Launcher installs the executable and creates the data and settings files.
For development, `dev_installer.sh`/`dev_installer.ps1` do the same (without overwriting
existing files, unless `--reset`/`-Reset` is given).

### Usage

* `--install PATH NAME` => Installs the mod from the archive/folder with the name NAME
  (`--overwrite` replaces the files of conflicting mods, `--verbose` prints every step)
* `--uninstall NAME` => Uninstalls the mod with the name NAME
* `--list-mods` => Lists all installed mods
* `--enable NAME` => Enables the mod with the name NAME
* `--disable NAME` => Disables the mod with the name NAME
* `--settings NAME VALUE` => Sets the setting NAME to VALUE
* `--list-settings` => Lists all settings
* `--automata` => Starts NieR:Automata
* `--files` => Lists ATA's files and folders
* `--detect-game` => Looks for NieR:Automata in the Steam libraries
* `--wipe` => Uninstalls every mod

If you don't remember flags just run `ata --help`.
If you want more details about usage run `cargo doc --no-deps --open`

### Output (for GUI authors)

- On success, the result is printed on **stdout** as JSON (a single-element array for commands
  acting on one mod or setting).
- Errors are printed on **stderr**, and so are warnings (lines starting with `warning:`).
- Exit codes:

| Code | Meaning |
| ---- | ------- |
| 0 | Success |
| 1 | The data or settings file couldn't be loaded (the Launcher can repair them) |
| 2 | Invalid arguments |
| 3 | The command failed |
| 4 | Installation refused because of conflicts: retry with `--overwrite` to replace the files |

### Using the library

```toml
[dependencies]
# Everything (installation included)
ata = { package = "ATA", path = "../backend" }
# Only paths, settings and mods data (no archive libraries)
ata = { package = "ATA", path = "../backend", default-features = false }
```

Enable the `specta` feature to derive `specta::Type` on every data type and generate
TypeScript bindings (e.g. with `tauri-specta`).

