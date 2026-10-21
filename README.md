
# pass os

An independent, device-aware operating system built on Linux, with Rust at its foundation.

## About

pass os explores a simple question:

What if Linux were approached with the attention to interaction, performance, integration, and consistency that defines thoughtfully designed operating systems?

The goal is not to reproduce macOS, iOS, ChromeOS, or another existing system.

pass os takes inspiration from their strengths while developing its own architecture, interaction model, security mechanisms, and user experience.

## Principles

- One adaptive operating-system platform.
- Device capabilities over predefined device categories.
- Rust-first system components.
- Responsive, consistent interaction.
- Security and privacy as architectural requirements.
- Clear separation of responsibilities.
- Compatibility without unnecessary dependence.
- Modular, testable, maintainable code.

## Architecture

pass os uses the Linux kernel for hardware support, scheduling, memory management, and process isolation.

Its graphical environment, device-awareness layer, authentication components, and system interfaces are developed independently.

Initially, Debian provides access to an established package ecosystem.

The long-term goal includes native application distribution and a unified software experience.

Mobile support may incorporate components from the Android Open Source Project while sharing appropriate higher-level pass os components.

## Repository

```text
crates/
├── compositor/   Wayland compositor
├── device/       Hardware capability discovery
├── pam/          PAM integration
├── passid/       Authentication library
└── passidd/      Authentication daemon
```

Additional components will be introduced as their responsibilities become clear.

## Current status

Early development.

Completed:

- Rust workspace established.
- Initial passid passcode and lockout implementation.
- Initial device capability discovery.
- Smithay integration.
- Nested compositor initialization.
- Initial OpenGL ES background rendering.
- ARM64 development environment established.

Not yet implemented:

- Complete Wayland server integration.
- Application surface and window management.
- Adaptive graphical shell.
- Production authentication integration.
- Native display backend.
- Installation, updates, and recovery.

## Development

Primary development hardware:

- Orange Pi 5 Plus
- ARM64
- Debian 13 (Trixie)
- Armbian vendor kernel
- Rust 2024 edition

See [Development](docs/development.md) for build and testing instructions.

See [Architecture](docs/architecture.md) for system design.

## Licensing

The Rust workspace currently declares GPL-3.0-or-later.

Individual components and third-party dependencies remain subject to their applicable licenses.
