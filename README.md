![Plugin Icon](assets/icon.png)

# OpenDeck Redragon Streamcraft SS550 Plugin

An unofficial plugin for Redragon Streamcraft SS550

## OpenDeck version

Requires OpenDeck 2.5.0 or newer

## Supported devices

- Redragon Streamcraft SS550 (0200:1000)

## Platform support

- Linux: Guaranteed, if stuff breaks - I'll probably catch it before public release
- Mac: Best effort, no tests before release, things may break, but I probably have means to fix them
- Windows: Zero effort, no tests before release, if stuff breaks - too bad, it's up to you to contribute fixes

## Installation

1. Download an archive from [releases](https://github.com/MMonkeyKiller/opendeck-ss550/releases)
2. In OpenDeck: Plugins -> Install from file
3. Linux: Download [udev rules](./40-opendeck-ss550.rules) and install them by copying into `/etc/udev/rules.d/` and running `sudo udevadm control --reload-rules`
4. Unplug and plug again the device, restart OpenDeck

## Known issues

- All the "old" devices come with the same serial number. You cannot use two of the same devices at the same time (for example a pair of 153R-s), but you can use two different devices at the same time (for example a 153R and a 153E)

## Building

### Prerequisites

You'll need:

- A Linux OS of some sort
- Rust 1.87 and up with `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-gnu` targets installed
- Docker
- [just](https://just.systems)

### Preparing environment

```sh
$ just prepare
```

This will build docker image for macOS crosscompilation

### Building a release package

```sh
$ just package
```

## Acknowledgments

This plugin is heavily based on work by contributors of [elgato-streamdeck](https://github.com/streamduck-org/elgato-streamdeck) crate
