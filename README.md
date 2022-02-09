# Plutonium Updater CLI
#### Stop uploading, start downloading
###### (So catchy!)

![](https://screen.sbs/i/2133v3q6.png)

### Features
- Multi-platform
- Version checking
- File hash checking

### Usage

- Download the latest release
  - [Linux](https://github.com/mxve/plutonium-updater.rs/releases/latest/download/plutonium-updater-x86_64-unknown-linux-gnu.tar.gz)
    - ```https://github.com/mxve/plutonium-updater.rs/releases/latest/download/plutonium-updater-x86_64-unknown-linux-gnu.tar.gz```
  - [Windows](https://github.com/mxve/plutonium-updater.rs/releases/latest/download/plutonium-updater-x86_64-pc-windows-msvc.zip)
    - ```https://github.com/mxve/plutonium-updater.rs/releases/latest/download/plutonium-updater-x86_64-pc-windows-msvc.zip```
  - [MacOS](https://github.com/mxve/plutonium-updater.rs/releases/latest/download/plutonium-updater-x86_64-apple-darwin.tar.gz) (untested)
    - ```https://github.com/mxve/plutonium-updater.rs/releases/latest/download/plutonium-updater-x86_64-apple-darwin.tar.gz```
- Unpack archive
- Run it
  - Preferably from a terminal so you can see the output and append the arguments listed below, if needed.
  - Linux
    - Unpack
      - ```tar xfv plutonium-updater-x86_64-unknown-linux-gnu.tar.gz```
    - Make binary executable
      - ```chmod +x plutonium-updater```
    - Run it
      - ```./plutonium-updater```
  - Windows
    - Unpack
    - Run it

### Examples
##### Windows update .bat
```
@echo off
set installDir=C:\your_pluto_directory
plutonium-updater.exe -d "%installDir%"
```

##### Linux update .sh
```
#!/bin/bash
INSTALLDIR=/home/pluto/pluto_dir
./plutonium-updater.exe -d "$INSTALLDIR"
```

##### Repair files
```
./plutonium-updater.exe -f
```
or
```
./plutonium-updater.exe -fd "pluto directory"
```

### Arguments
- ```-d, --directory <path>```
  - Install directory, supports relative and absolute paths
  - Default: "plutonium"
- ```-f, --force```
  - Force file hash re-check even if version matches
- ```-l, --launcher```
  - Download launcher assets.
- ```-q, --quiet```
  - Hide file actions (Checked, Skipped, Downloaded)
- ```-s, --silent```
  - Hide all non-error output
- ```-c, --check```
  - Compares local version to remote
    - Exit code 0 for up to date
    - Exit code 1 for outdated

### Exit codes
- 0 success
- 101 error (rust panic)
(Just fail on everything that differs from 0 if you are scripting it)
