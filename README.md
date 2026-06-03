# Accord's Timeline Alterer (ATA)

ATA is a mod manager for NieR:Automata for both Linux and Windows.

## Features
- Install mods
- Uninstall mods
- List mods you have installed
- Enabling/Disabling mods
- Managing some settings/preferences

## Installation/Usage
### Installation
1. Copy the repo `git clone https://github.com/AccordDev/Accord-Timeline-Alterer.git`
2. cd into the repo `cd Accord-Timeline-Alterer`
3. Build the project `cargo build --release`
4. Run the executable `./target/release/ata`

### Usage
* --install /path/to/mod/archive NAME  => Installs the mod from the archive at /path/to/mod/archive with the name NAME
* --uninstall NAME  => Uninstalls the mod with the name NAME
* --list  => Lists all installed mods
* --enable NAME  => Enables the mod with the name NAME
* --disable NAME  => Disables the mod with the name NAME
* --settings NAME VALUE  => Sets the setting NAME to VALUE

If you don't remember flags just run `ata --help`.
If you want more details about usage run `cargo doc --no-deps --open`