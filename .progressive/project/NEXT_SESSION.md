# Next Session

Outcome: IN PROGRESS

## Current phase

Phase 01 — TLP Pro 1025M editor, Figma interaction core and GDL export

## Completed this session

- 2026-11-25 change request: a new project starts by choosing the touch panel (TLP Pro 1025M or 300M portrait), and the Artboard tool (Shift+O) and `extron.addPage` add pages. More than one page exports as an EXUI/5 project. Evidence is in the Phase 01 task list.
- 2026-11-25 change request: UI kits. Aurora, Daylight and Graphite are built in, and others install from `gui-studio-kit/1` files. A project keeps a copy of its kit. Blocks insert ready-made groups of controls.
- Reviewed the task-07 Problems panel work copied from the former fork checkout. Completed [task-07-r1](../phases/01-1025m-editor.prompts.md#task-07-r1).
- An Extron screen lists every export problem under the layers. Activating a problem selects its layer, reveals it in Layers and brings it into view. Blocked exports name each layer and node.
- The Warning severity and the page and popup rules moved to Phase 02, because they need pages and popups.

## Verification evidence

- 5 `gui-studio-extron` tests, 3 `extron_problems_tests` and the 28 i18n tests passed. Clippy with warnings denied and rustfmt passed for the three changed packages. The desktop window was not launched for a manual check.

## Current working state

RUNNABLE / GREEN. Not committed.

## Blockers / uncertainty

- The two-page 1025M and 300M GDLs passed the exporter's verifier but were not opened, verified or built in GUI Designer.
- The task-07 changes are in the working tree and not committed.
- The Rust toolchain and the MinGW and Cargo caches used for builds still live under `../gui-studio-fork/target` and `../figma-extron-exporter/output/rust-studio-toolchain`. Cargo is not on PATH. An environment script must set `RUSTUP_HOME`, `CARGO_HOME`, `PATH`, `LIBRARY_PATH` and `C_INCLUDE_PATH` to those locations before `cargo` runs. Deleting the fork checkout breaks local builds.

## Next action

Author Extron controls in the editor: commands for an empty 1025M panel and for inserting or assigning Button, Slider, Label and Shape, with IDs from a monotonic counter in the panel frame's `plugin_data`. See the Phase 01 task list.

## Active task prompt

None yet. Create task-08-r1 in `.progressive/phases/01-1025m-editor.prompts.md` for the control-authoring task.
