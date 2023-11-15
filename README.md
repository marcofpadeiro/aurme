# AURme

A simple AUR helper made in Rust.

## Description

AURme is a simple and minimalist tool made to help users install, update and maintain packages from the [AUR](https://aur.archlinux.org). AURme won't interact with pacman packages and is not meant to be a all-in-one package manager.

**!Important!**
The AUR is a great community driven repository with all type of packages and contributors, we highly encourage PKGBUILD reviews before installation because there can be some contributors with bad intentions spreading around malicious code.

## Features

- Installation and dependency resolution
- Search for most popular packages and allow selection
- AUR package updating
- AUR package cache cleaning

## Installation (WIP)

```
sudo pacman -S --needed cargo
git clone https://github.com/marcofpadeiro/aurme
cd aurme
cargo build --release
```

The executable will be localated at `target/release/aurme`, you can copy it or create a symmlink to `/usr/bin`

## Usage

| Commad | Description |
| ------ | ----------- |
|`aurme -S [AUR packages]`| Downloads and installs the specified AUR package(s) and their dependencies |
|`aurme -Ss <term>`| Searches for packages in the AUR and presents an installation menu |
|`aurme -Syu [AUR packages]`| Updates the specified AUR package, or updates all AUR packages if no specific package is provided |
|`aurme -Sc [AUR packages]`| Clears the cache for all AUR packages if no specific package is provided |

## Configuration

```json
{
  "cache_path": ".cache/aurme",
  "keep_cache": true,
  "no_confirm": false,
  "verbose": "default" // quiet, verbose
}
```

## Future

This project is still on a really early phase and there are still a lot of features that the devs intend to implement on the future, here are some of them, if you have any suggestions feel free to open an issue.

- Option to review PKGBUILD on install and update
- ~~Syntax colors and bold text~~
- ~~Improve flag handling and add more options like --verbose --quiet --ignore~~
- Check the PKGBUILD of a package before downloading
- Option to clone a package without building it
- Option to build a package stored without needing to download on the cache
- Option to build a package in the current directory
- On install print the required dependencies
- ~~Config file to setup preferences~~
- Pull latests changes of a package without building it
- Command line autocomplete for zsh bash and fish

![](https://media.tenor.com/Hw0aKasI6B4AAAAC/fast-blazing-fast.gif)