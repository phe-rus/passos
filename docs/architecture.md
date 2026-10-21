
# Architecture

## Overview

pass os is a Linux-based operating-system platform designed around hardware capabilities, independent system components, and an adaptive user experience.

Its architecture is intended to support different interaction environments without maintaining unrelated operating systems for every device class.

## Design principles

### Device awareness

Devices are described by their capabilities rather than fixed labels such as desktop, phone, tablet, or television.

Hardware discovery and interface adaptation are separate responsibilities.

The device subsystem reports available capabilities. Higher-level components determine how those capabilities should affect interaction.

### Separation of responsibilities

Each component should have a clear purpose and a narrow public interface.

Internal modules should remain simple until additional complexity is justified by actual requirements.

### Performance

Responsiveness must be measured.

Performance priorities include:

- Predictable input response.
- Efficient frame scheduling.
- Minimal unnecessary rendering.
- Avoiding blocking operations on critical paths.
- Controlled background resource consumption.
- Appropriate hardware acceleration.

Rust provides useful memory-safety guarantees but does not automatically guarantee performance.

### Security

Security should be designed into component boundaries.

Privileged services must expose limited, authenticated interfaces.

Graphical applications and shell processes should not require unrestricted system privileges.

## Components

### passid

Authentication and identity subsystem.

Existing functionality:

- Argon2id passcode hashing.
- Passcode verification.
- Lockout state management.
- Escalating lockout delays.

The current implementation is a prototype, not a complete production authentication system.

### passidd

Authentication daemon.

Currently contains a command-line demonstration of passid behavior.

Future responsibilities include authenticated IPC, secure credential handling, account integration, and coordination with session management.

### pam

Linux PAM integration.

Currently a placeholder implementation.

Production integration requires explicit threat modeling and testing.

### compositor

Rust-based Wayland compositor using Smithay.

Development backend: winit.

Current functionality:

- Initializes the nested graphics backend.
- Creates a window.
- Handles resize events.
- Initializes an OpenGL ES renderer.
- Clears and submits a background frame.

Future functionality:

- Wayland protocol handling.
- Application surface management.
- Window focus, movement, and resizing.
- Input routing.
- Frame scheduling and damage tracking.
- Native DRM/KMS integration.
- Multiple displays.

Smithay is an implementation dependency, not the definition of pass os.

### device

Hardware capability discovery.

Current implementation inspects Linux DRM connectors through sysfs.

Future responsibilities include:

- Display capabilities.
- Input devices.
- Power and battery information.
- Relevant sensors.
- Connectivity capabilities.
- Hardware change notifications.

Device reporting should distinguish unknown, unsupported, unavailable, and disconnected capabilities where appropriate.

## Platform integration

Linux is the initial underlying system.

Debian repositories provide early application and package compatibility.

Hardware-specific bootloaders, device trees, firmware, and kernel drivers may still be necessary.

Future mobile integration may use Android Open Source Project components for hardware and application compatibility.

## Naming conventions

The canonical ecosystem name is `pass`.

Technical identifiers use lowercase.

Examples:

- `passid`
- `passidd`
- `pass-compositor`
- `pass-device`

Repository directories use short responsibility-based names.

Examples:

- `crates/compositor`
- `crates/device`
- `crates/pam`

Rust naming follows standard language conventions.

## Architectural decisions

- ADR-001: Linux as the initial kernel.
- ADR-002: Rust as the primary system implementation language.
- ADR-003: Capability-driven device adaptation.
- ADR-004: Wayland and Smithay for initial compositor development.
- ADR-005: Debian package compatibility during early development.

These decisions should be revisited when technical evidence justifies changes.
