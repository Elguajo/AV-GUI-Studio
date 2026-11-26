# Next Session

Outcome: IN PROGRESS

## Current phase

Phase 01 — TLP Pro 1025M editor, Figma interaction core and GDL export

## Completed this session

- 2026-11-25 change request: a new project starts by choosing the touch panel (TLP Pro 1025M or 300M portrait), and the Artboard tool (Shift+O) and `extron.addPage` add pages. More than one page exports as an EXUI/5 project. Evidence is in the Phase 01 task list.
- 2026-11-25 change request: UI kits. Aurora, Daylight and Graphite are built in, and others install from `gui-studio-kit/1` files. A project keeps a copy of its kit. Blocks insert ready-made groups of controls.
- Completed [task-07-r1](../phases/01-1025m-editor.prompts.md#task-07-r1): the Problems panel (committed in 1c380be).
- Completed [task-08-r1](../phases/01-1025m-editor.prompts.md#task-08-r1): buttons and sliders are instances of one master component per kit beside the pages. Editing a master restyles every control made from it; captions, sizes, keys and IDs stay with each instance. A problem in a master is reported once, at the master.

## Verification evidence

- Task-08: the full `gui-studio-extron` (19), `figcraft-engine` (2283 + 51), `figcraft-ui-egui` (1357 + 12) and `figcraft-mcp` suites passed, with Clippy (warnings denied) and rustfmt. The demo EXUI stayed `2B474E44…`. The kits project EXUI stayed byte-identical (`11959BBF…`, GDL 294/294), and a master-based project built a GDL with 131/131 checks. Artifacts are in the local `target/masters-evidence/`.

## Current working state

RUNNABLE / GREEN. Task-08 is not committed.

## Blockers / uncertainty

- No GDL from pages, kits or masters has been opened, verified or built in GUI Designer, and the desktop window was not launched to try master editing by hand.
- The Rust toolchain and the MinGW and Cargo caches used for builds still live under `../gui-studio-fork/target` and `../figma-extron-exporter/output/rust-studio-toolchain`. Cargo is not on PATH. An environment script must set `RUSTUP_HOME`, `CARGO_HOME`, `PATH`, `LIBRARY_PATH` and `C_INCLUDE_PATH` to those locations before `cargo` runs. Deleting the fork checkout breaks local builds.

## Next action

Port the identity tests from the eframe prototype: restyle, reorder and reopen keep Mute's key and IDs, and a replacement after delete gets a new ID. Restyle Mute through its master. See the Phase 01 task list.

## Active task prompt

None yet. Create task-09-r1 in `.progressive/phases/01-1025m-editor.prompts.md` for the identity tests.
