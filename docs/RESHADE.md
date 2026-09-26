# ReShade presets in ATA

This document collects how ReShade presets are distributed for NieR:Automata and how ATA manages them.

## Known facts

- The latest ReShade version tested to work with every preset is **4.7.0**.
- ReShade itself (its DLL and addons) is **not** installed by ATA.
- Presets come as a lone `.ini` file, or as an `.ini` plus folders of custom effects.

## How ReShade 4.x is laid out in the game folder

NieR:Automata is a DirectX 11 game. ReShade is installed next to `NieRAutomata.exe`:

```text
NieRAutomata/
├── NieRAutomata.exe
├── dxgi.dll                 <- ReShade itself (can also be named d3d11.dll)
├── ReShade.ini              <- ReShade's configuration
├── MyPreset.ini             <- a preset
└── reshade-shaders/
    ├── Shaders/             <- effects (.fx) and headers (.fxh)
    └── Textures/            <- textures used by effects (.png, .jpg, .dds...)
```

`ReShade.ini` (section `[GENERAL]`) contains, among others:

- `EffectSearchPaths` (default `.\reshade-shaders\Shaders`): where effects are looked for
- `TextureSearchPaths` (default `.\reshade-shaders\Textures`): where textures are looked for
- `PresetPath`: the preset currently in use

A **preset** is an `.ini` file that lists the effects to turn on and their parameters:

```ini
Techniques=SMAA@SMAA.fx,Clarity@Clarity.fx
TechniqueSorting=SMAA@SMAA.fx,Clarity@Clarity.fx

[Clarity.fx]
ClarityStrength=0.400000
```

## How presets are distributed

Preset archives (e.g. on Nexus Mods) typically come in one of these shapes, and ATA handles all of them:

1. **Only the preset**: `MyPreset.ini`. It relies on the effects shipped with ReShade
   (the standard effects chosen during the ReShade setup).
2. **Preset and effects**: `MyPreset.ini` plus a `reshade-shaders/` folder (sometimes just
   `Shaders/` and `Textures/`, or loose `.fx` files) with the custom effects the preset needs.
3. **Preset and ReShade**: some archives also bundle a ReShade build (`dxgi.dll`,
   `ReShade.ini`), sometimes a different version from the one the user has.

Many presets ship the same common files (`ReShade.fxh`, `ReShadeUI.fxh`, popular effects),
usually with identical contents.

## What ATA does

Installation (`src/features/installation/reshade.rs`):

- A mod is a ReShade preset when it contains an `.ini` file with a `Techniques=` or
  `TechniqueSorting=` key. The content is checked because any mod can ship `.ini` files
  (e.g. configuration of DLL mods), and those must not be mistaken for presets.
- Presets go to the game folder, next to the executable.
- Effects and textures go to `reshade-shaders/Shaders` and `reshade-shaders/Textures`,
  keeping their subfolders. These are the default search paths of ReShade 4.x, so no
  configuration change is needed.
- ReShade's own files (`dxgi.dll`, `d3d11.dll`, ..., `ReShade.ini`, logs) are skipped, with a
  warning. Installing them would silently replace the ReShade version the user has.
- Screenshots, readmes and anything else are ignored.
- If ReShade doesn't look installed (no `ReShade.ini` plus ReShade DLL in the game folder),
  the installation still succeeds, with a warning that the preset won't have any effect.

Shared files:

- If a shader of a new preset is identical to one already installed by another preset, it is
  **shared** instead of reported as a conflict. It is deleted only when the last preset using
  it is uninstalled.
- A shader with different contents is a regular conflict: it is reported and replaced only if
  the user chooses to overwrite.

Enabling/disabling:

- Only the preset `.ini` is moved to/from `.disabled/`. Effects stay in place: they are
  often shared, and ReShade only runs the techniques listed in the active preset.

## Possible next steps

- **Activating a preset**: set `PresetPath` in `ReShade.ini` when a preset is enabled
  (and restore the previous value when it is disabled). It was left out on purpose: it
  changes ReShade's configuration, which ATA doesn't own yet.
- **Custom search paths**: presets for other ReShade setups expect different folders;
  `EffectSearchPaths`/`TextureSearchPaths` could be read from `ReShade.ini` to install
  effects where the user's ReShade looks for them.
- **Installing ReShade**: once ATA manages ReShade itself, the ReShade files currently
  skipped could be offered as a separate "ReShade" mod, pinned to version 4.7.0.
