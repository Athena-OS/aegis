# Contribution Guidelines

This doc is designed to give a high level overview of how this codebase works, to make contributing to it easier.

## Setting up the dev environment

The flake included in the project root contains a dev shell which will give you all of the tools you need to work on the project. If you're on NixOS or have `nixpkgs` installed on your machine, you can just use
```bash
nix develop
```

If not, make sure you have cargo installed. Also, run cargo fmt before you make any commits please :)

## nixos-wizard Architecture Overview

The program itself has five core components:

1. The event loop - manages current UI and installer state
2. The `Page` trait - defines the main UI screens, essentially containers for widgets
3. The `ConfigWidget` trait - re-usable UI components that make up pages
4. The `Installer` struct - contains all of the information input by the user
5. The `nixgen` module - responsible for serializing the `Installer` struct into a `configuration.nix` file

### The event loop
The event loop contains a stack of `Box<dyn Page>`, and whenever a page is entered, that page is pushed onto the stack. Whenever a page is exited, that page is popped from the stack. Every iteration of the event loop does two things:
* Calls the `render()` method of the page on top of the stack
* Polls for user input, and if any is received, passes that input to the `handle_input()` method of the page on top of the stack.
The pages communicate with the event loop using the `Signal` enum. `Signal::Pop` makes the event loop pop from the page stack, for instance.

### The `Page` trait
The `Page` trait is the main interface used to define the different pages of the installer. The main methods of this trait are `render()` and `handle_input()`. Each page is itself a collection of widgets, which each implement the `ConfigWidget` trait. Pages are navigated to by returning `Signal::Push(Box::new(<page>))` from the `handle_input()` method, which tells the event loop to push a new page onto the stack. Pages are navigated away from using `Signal::Pop`.

### The `ConfigWidget` trait
The `ConfigWidget` trait is the main interface used to define page components. Like `Page`, the `ConfigWidget` trait exposes `render()` and `handle_input()`. `handle_input()` is useful when input *must* be passed to the widget using the interface, like in the case of said widget being stored as a trait object. `render()` is usually given a chunk of the screen by it's `Page` to try to render inside of.

Generally speaking, inputs are caught and handled at the page level, as delegating all input to the individual widgets ends up fostering more presumptuous or general logic, where page-specific logic is generally more favorable in this case.

The trickiest part of setting up new `Page` or `ConfigWidget` structs is defining how they use the space that they are given in their respective `render()` methods. Take this for example:

```rust
impl Page for EnableFlakes {
  fn render(&mut self, _installer: &mut Installer, f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .margin(1)
      .constraints(
				[
					Constraint::Percentage(40),
					Constraint::Percentage(60)
				].as_ref()
			)
      .split(area);

    let hor_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .margin(1)
      .constraints(
        [
          Constraint::Percentage(30),
          Constraint::Percentage(40),
          Constraint::Percentage(30),
        ]
        .as_ref(),
      )
      .split(chunks[1]);

    let info_box = InfoBox::new(
      "",
      ... info box content ...
    );
    info_box.render(f, chunks[0]);
    self.buttons.render(f, hor_chunks[1]);
    self.help_modal.render(f, area);
  }
...
```

This is the `render()` method of the "Enable Flakes" page. It cuts up the space given to it vertically first, and then horizontally.

The method uses Ratatui's `Layout` system to divide the terminal screen area into smaller rectangular chunks. First, it splits the available space vertically into two regions: the top 40% (for the `info_box`) and the bottom 60%. Then it subdivides the bottom 60% horizontally into three parts: 30%, 40%, and 30%. The middle horizontal chunk is used to render the `buttons` widget.

Each widget’s `render()` method is called with the frame and the specific chunk of the terminal space it should draw itself within. This way, each widget knows exactly how much space it has, and where it should be positioned on the screen.

This approach of dividing and subdividing the UI space using Ratatui’s layout tools allows pages to arrange their child widgets precisely and responsively, adapting to terminal size changes.

## The `Installer` Struct

The `Installer` struct (defined in `src/installer/mod.rs`) serves as the central data store for all user configuration choices throughout the installation process. It acts as the single source of truth that gets populated as users navigate through different pages and make selections.

### Key Fields

The struct contains fields for every configurable aspect of a NixOS installation:

**System Configuration:**
- `hostname`, `timezone`, `locale`, `language` - Basic system settings
- `keyboard_layout` - Keyboard layout configuration
- `enable_flakes` - Whether to enable Nix flakes support
- `bootloader` - Boot loader choice (e.g., "systemd-boot", "grub")

**Hardware & Storage:**
- `drives` - Vector of `Disk` objects representing storage configuration
- `use_swap` - Whether to enable swap partition
- `kernels` - Available kernel options
- `audio_backend` - Audio system configuration

**User Management:**
- `root_passwd_hash` - Hashed root password
- `users` - Vector of `User` structs containing user account information

**Desktop Environment:**
- `desktop_environment` - Selected DE (e.g., "KDE Plasma", "GNOME")
- `greeter` - Display manager choice
- `profile` - Installation profile selection

**Packages & Services:**
- `system_pkgs` - Vector of system packages to install
- `network_backend` - Network management system
- `flake_path` - Optional path to user's flake configuration

### Key Methods

The `Installer` struct provides several important methods:

- `new()` - Creates a new instance with default values
- `has_all_requirements()` - Validates that all required fields are populated before installation can proceed. Checks for root password, at least one user, drive configuration, and bootloader selection.

### Usage Pattern

Throughout the application, pages and widgets receive a mutable reference to the `Installer` struct, allowing them to read current values and update fields based on user input. This centralized approach ensures data consistency and makes it easy to validate the complete configuration before proceeding with installation.

## The `nixgen` Module

The `nixgen` module (located in `src/nixgen.rs`) is responsible for converting the user's configuration choices stored in the `Installer` struct into valid Nix configuration files. This module serves as the bridge between the TUI application and the actual NixOS configuration system.

### Core Components

**NixWriter Struct:**
The main component is the `NixWriter` struct, which takes a JSON `Value` representation of the configuration and provides methods to generate different types of Nix configuration files.

**Key Functions:**
- `nixstr(val)` - Utility function that wraps strings in quotes for valid Nix syntax
- `fmt_nix(nix)` - Formats generated Nix code using the `nixfmt` tool
- `highlight_nix(nix)` - Syntax highlights Nix code using `bat` for display purposes
- `attrset!` - A macro that allows you to write Nix attribute sets. Returns a `String`.

### Configuration Generation

The `NixWriter` generates two main types of configuration:

**System Configuration (`write_sys_config`):**
- Converts user choices into a complete `configuration.nix` file
- Handles system-level settings like networking, desktop environments, users, and services
- Uses helper functions like `parse_network_backend()` and `parse_locale()` to convert user-friendly selections into proper Nix attribute sets
- Manages conditional logic for features like Home Manager integration

**Disko Configuration (`write_disko_config`):**
- Generates disk partitioning and filesystem configuration
- Creates the declarative disk setup that Disko will execute during installation
- Handles different storage configurations based on user selections

### Output Structure

The `write_configs()` method returns a `Configs` struct containing:
- `system` - The complete NixOS system configuration as a Nix string
- `disko` - The disk configuration for the Disko tool
- `flake_path` - Optional path to user's existing flake configuration

### Architecture Benefits

This separation allows the installer to:
1. Maintain a clean separation between UI logic and configuration generation
2. Generate human-readable, properly formatted Nix configurations
3. Support both traditional NixOS configurations and flake-based setups
4. Provide immediate validation and preview of the generated configurations before installation
