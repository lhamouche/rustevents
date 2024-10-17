# rustevents

A Rust-based Linux devices input events reader.

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
- [Arguments](#arguments)
- [Examples](#examples)

## Features

- Reads input events from specified Linux input device files (e.g., `/dev/input/eventX` or dumps).
- Supports customizable keyboard layouts, models, rules, and locales.
- Handles key states and compose sequences for multi-character input.
- Outputs UTF-8 characters to standard output for real-time interaction.

## Requirements

- **Rust**.
- **xkbcommon**: Development libraries for keyboard handling.

### On Debian/Ubuntu

You can install the necessary xkbcommon library with the following command:

```bash
sudo apt-get install libxkbcommon-dev
```

## Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/lhamouche/rustevents
   cd rustevents
   ```

2. **Build the project**:

   ```bash
   cargo build --release
   ```

3. **Run the compiled binary** located in the `target/release/` directory.

## Usage

```bash
./target/release/rustevents [OPTIONS] --file <FILE>
```

## Arguments

| Argument  | Short | Default         | Description                                                 |
|-----------|-------|------------------|-------------------------------------------------------------|
| `file`    | `-f`  | *(required)*     | The path to the input device file (e.g., `/dev/input/eventX`). |
| `rules`   | `-r`  | `evdev`          | The rules for the keyboard layout (e.g., `evdev`, `xorg`). |
| `model`   | `-m`  | `pc105`          | The keyboard model.                                         |
| `layout`  | `-l`  | `us`             | The keyboard layout (e.g., `us`, `fr`).                    |
| `variant` | `-v`  | `""`             | Optional layout variant.                                   |
| `locale`  | `-c`  | `en_US.UTF-8`    | The locale for compose sequences.                          |

## Examples

1. **Read input events from a keyboard** connected to `/dev/input/event0`:

   ```bash
   ./target/release/rustevents --file /dev/input/event0
   ```

2. **Specify a different layout ,variant and locale**:

   ```bash
   ./target/release/rustevents --file event0 --layout fr --variant latin9 -c fr.UTF-8
   ```