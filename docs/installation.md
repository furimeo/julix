# Julix Installation

Julix is distributed as a zip archive, installed via a script.

## Install

### Linux / macOS

```sh
curl -sSf https://raw.githubusercontent.com/furimeo/julix/main/scripts/install.sh | sh
```

### Windows (PowerShell)

```ps1
irm https://raw.githubusercontent.com/furimeo/julix/main/scripts/install.ps1 | iex
```

## What the install script does

1. Download the zip for the current platform from GitHub releases.
2. Extract into `~/.julix` (Linux/macOS) or `%USERPROFILE%\.julix` (Windows).
3. Add `~/.julix/bin` to PATH.
4. Create symlinks or hardlinks:
   - `lixvm` points to `julix`
   - `jujit` points to `julix`

## Layout after install

### Linux / macOS

```
~/.julix/
├── bin/
│   ├── julix              // main binary
│   ├── lixvm -> julix     // symlink
│   └── jujit -> julix     // symlink
└── lib/
    └── stdlib/            // standard library (later)
```

### Windows

```
%USERPROFILE%\.julix\
├── bin\
│   ├── julix.exe          // main binary
│   ├── lixvm.exe          // hardlink to julix.exe
│   └── jujit.exe          // hardlink to julix.exe
└── lib\
    └── stdlib\            // standard library (later)
```

## Verify

```sh
julix --version
# Julix 0.0.2
# LixVM 0.0.0
# JuJIT 0.0.0

lixvm --version
# LixVM 0.0.0

jujit --version
# JuJIT 0.0.0
```
