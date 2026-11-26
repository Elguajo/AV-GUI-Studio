# Vendored egui-winit

Source: `crates/egui-winit` of [storytold/egui](https://github.com/storytold/egui) at
`6d518fc799681571963842f95b55a6b68431b15a`, the commit the rest of the egui family is pinned to in
the root `Cargo.toml`. Licence: MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`).

It sits where that commit keeps `egui-winit` inside the vendored egui (`vendor/egui`), so eframe's
own dependency on it resolves here, and the root `Cargo.toml` patches crates.io's `egui-winit` to it.

## Changes from the source

- `src/lib.rs`: `WindowEvent::Moved` no longer requests a repaint. On Windows every step of a
  title-bar drag otherwise drew and presented a full frame inside the system's move loop (60 frames/s
  on a 60 Hz display), and the window trailed the cursor. Upstream: emilk/egui#8946, emilk/egui#8947.
- `Cargo.toml`: workspace-inherited fields written out; dev-dependencies and examples dropped.

## Updating

When `vendor/egui` moves to another commit, copy that commit's `crates/egui-winit/src` here again
and re-apply the change above, or take that commit's copy unchanged once it has the fix.
