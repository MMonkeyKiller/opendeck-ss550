![Plugin Icon](assets/icon.png)

# OpenDeck Ajazz AKP153 / Mirabox HSV293 Plugin

An unofficial plugin for Mirabox HSV293-family devices

## OpenDeck version

Requires OpenDeck 2.5.0 or newer

## Supported devices

- Mirabox HSV293S (5548:6670)
- Ajazz AKP153 (5548:6674)
- Ajazz AKP153E (0300:1010)
- Ajazz AKP153R (0300:1020)

## Installation

1. Download an archive from [releases](https://github.com/4ndv/opendeck-akp153/releases)
2. In OpenDeck: Plugins -> Install from file
3. Download [udev rules](./40-opendeck-akp153.rules) and install them by copying into `/etc/udev/rules.d/` and running `sudo udevadm control --reload-rules`
4. Unplug and plug again the device, restart OpenDeck

## Acknowledgments

This plugin is heavily based on work by contributors of [elgato-streamdeck](https://github.com/streamduck-org/elgato-streamdeck) crate
