
# Development

## Last development checkpoint

Date: 2026-10-21

Status: Initial graphics backend and background rendering compile and run successfully.

## Development environment

Hardware: Orange Pi 5 Plus

Operating system: Debian 13 (Trixie), Armbian

Kernel: 6.1.115-vendor-rk35xx

Architecture: aarch64

Memory: approximately 8 GB

Repository:

`/root/platforms/passos`

Remote graphical testing uses TigerVNC with a minimal Openbox desktop.

A VNC viewer connects from ChromeOS.

## Important environment details

The VNC desktop uses display `:1`.

The current OpenGL renderer is Mesa llvmpipe.

This means rendering inside VNC is software-based.

The Orange Pi's Mali-G610 GPU is initialized through the vendor `mali_kbase` driver, but GPU acceleration has not been established for the development compositor.

Do not assume that VNC performance represents native graphics performance.

## Rust workspace

The workspace uses:

- Rust edition 2024.
- Cargo resolver 3.
- GPL-3.0-or-later workspace licensing.

Smithay is currently resolved from Git revision:

`24647fc8d00131eb7f388efed26e35ea316b0018`

The Cargo lockfile records dependency resolution.

The Smithay reference implementation was tested separately at:

`/tmp/smithay-passos-test`

This temporary directory is not part of the pass os repository and may disappear.

## Build and test

From the repository root:

```bash
cd ~/platforms/passos

cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

At the last complete workspace verification:

- 11 unit tests passed.
- Strict Clippy passed.
- Formatting completed.

The PAM test is currently a placeholder.

## Run the compositor

The development desktop must be running.

```bash
cd ~/platforms/passos

DISPLAY=:1 cargo run -p pass-compositor
```

Current behavior:

- Opens a 1280×800 window by default.
- Processes window resize events.
- Initializes the GLES renderer.
- Clears the framebuffer to a dark background.
- Submits the rendered frame.

The EGL initialization may emit DRI2 warnings under TigerVNC.

The last execution successfully initialized the renderer and entered the event loop.

## Run device inspection

```bash
cargo run -p pass-device --bin inspect
```

Previously observed DRM connectors:

- card0-DP-1: disconnected
- card0-HDMI-A-1: disconnected
- card0-HDMI-A-2: disconnected
- card0-Writeback-1: unknown

The writeback connector should eventually be classified separately from physical display outputs.

## Run authentication demonstration

```bash
cargo run -p passidd
```

The current demonstration exercises passcode verification and lockout behavior.

It contains a hardcoded demonstration passcode and prints its generated hash.

Do not treat it as a production authentication daemon.

## Current compositor implementation

Location:

`crates/compositor/src/main.rs`

Implemented:

1. Smithay winit initialization.
2. Host window creation.
3. Window event processing.
4. OpenGL ES framebuffer rendering.
5. Background clearing.
6. Frame submission.

Limitations:

- No Wayland display server.
- No connected Wayland clients.
- No application surfaces.
- No window management.
- No independent desktop shell.
- No native DRM/KMS backend.
- Temporary frame scheduling.

## Next milestone

Implement the Wayland server inside `pass-compositor`.

The next implementation should:

1. Create a Wayland display.
2. Bind a uniquely named runtime socket.
3. Initialize essential compositor protocol state.
4. Accept and dispatch clients.
5. Implement xdg-shell toplevel management.
6. Render a Wayland application surface.
7. Route keyboard and pointer input.

Acceptance criterion:

A Wayland terminal, such as `foot`, appears inside our own pass compositor and can receive input.

## Development precautions

- Do not replace the working Armbian installation.
- Do not change the kernel or GPU overlays without a recovery plan.
- Do not run production sessions as root.
- Do not expose the VNC server to the public internet.
- Preserve passid functionality during refactors.
- Run tests and Clippy before committing.
- Keep the architecture device-aware rather than hardcoding device categories.
