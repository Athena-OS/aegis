<h1 align="center">nixos-wizard</h1>
<h6 align="center">
  A modern terminal-based NixOS installer, inspired by Arch Linux’s
  <a href="https://github.com/archlinux/archinstall">archinstall</a>.
</h6>

![nixos-wizard screenshot](https://github.com/user-attachments/assets/b1e11874-a72d-4e54-b2d8-e5a5f3325ac9)

---

## Why nixos-wizard?

NixOS is an amazing distribution, but manual installations from a terminal have always been a tedious and error prone process from day one. Many tools have surfaced that can reliably automate this process, but the options for manual installations are scarce.

This project aims to help you get a bootable NixOS system as quickly and easily as possible by providing:

* A **text-based UI** for an intuitive installation experience
* Interactive **disk partitioning and formatting** powered by [Disko](https://github.com/nix-community/disko)
* Guided **user account creation**, **Home Manager configuration**, and **package selection**
* Automatic **NixOS config generation**
* Real-time progress feedback during installation

---

## Features

* **Terminal UI** built with [Ratatui](https://github.com/ratatui/ratatui)
* Partition disks and create filesystems easily
* Configure users, groups, passwords, and even setup Home Manager.
* Select system packages to install
* Automatically generate and apply hardware-specific NixOS configurations
* Supports installation inside a NixOS live environment (recommended)

---

## Requirements & Recommendations

* Must be run **as root**.
* Designed to run inside the **NixOS live environment** built from the project’s flake or ISO. A prebuilt installer ISO is included with each release.
* Depends on NixOS-specific tools like `nixos-install` and `nixos-generate-config` being available.
* A terminal emulator with proper color and Unicode support is recommended for best experience.
* Running the binary directly may cause failures if necessary commands are not found in your environment. Ideally, this should be run using the flake output which wraps the program with all of the commands it needs for the installation process.

---

## Getting Started

### Development & Building

Use Nix flakes to enter the dev shell or build the project:

```bash
# Enter development shell with all dependencies
nix develop

# Build the release binary
nix build
```

### Running nixos-wizard

If running inside the included installer ISO:

```bash
sudo nixos-wizard
```

Alternatively, run the latest release from GitHub via Nix:

```bash
sudo nix run github:km-clay/nixos-wizard
```

---

## Building & Using the Installer ISO

You can build a custom NixOS ISO image that includes `nixos-wizard` and all its dependencies pre-installed:

```bash
nix build github:km-clay/nixos-wizard#nixosConfigurations.installerIso.config.system.build.isoImage
```

Boot this ISO on your target machine to run the installer in a fully-supported live environment.

---

## Roadmap

* Add support for **btrfs subvolumes** and snapshots in disk configuration
* Enable importing existing **flake inputs** or `configuration.nix` files for advanced customization
* Improve hardware detection and configuration automation
