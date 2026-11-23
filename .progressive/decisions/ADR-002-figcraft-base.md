# ADR-002 — GUI Studio on top of FigCraft

Status: Accepted
Date: 2026-11-23
Supersedes: the editor implementation in [ADR-001](ADR-001-workspace-canvas.md). The Brief's experience and visual-language principles still apply.

## Context

The Brief requires Figma interaction parity for our version of Extron GUI Designer. The in-house eframe/egui 0.33 editor would need years of work to reach that level. [FigCraft](https://github.com/storytold/figcraft) is an open-source Rust design app licensed MIT OR Apache-2.0, at about 109k lines and early-alpha status. It already provides a UI3-style workspace, auto layout, components and variants, variables, prototyping, unlimited undo, a command registry shared by menus, shortcuts, CLI and MCP, and per-node `plugin_data`. Its UI crate `figcraft-ui-egui` runs on the FigCraft engine and a storytold fork of egui 0.37, so it cannot be lifted onto the GUI Studio model.

## Decision

- **Fork as base.** GUI Studio becomes a downstream fork of FigCraft that tracks the `upstream` remote. GUI Studio's own work stays in separate crates and modules so upstream merges stay small. The fork is not published to GitHub until the user approves.
- **Extron as a native crate.** The new crate `crates/extron` holds panel profiles, the Extron control model, validation diagnostics, and the EXUI/5 writer. The writer is ported from today's `src/project.rs` and `src/exui.rs`. Engine commands under an `extron.*` namespace run validation, EXUI export and GDL build through the existing exporter CLI. WebAssembly plugins cannot be used for this, because their sandbox has no file system and cannot run processes.
- **Metadata on nodes.** Extron identity and roles live in node `plugin_data`, following the `exui-node/1` contract of Figma Extron Exporter: semantic key, Object ID, Control ID, role, and stateId for each state.
- **Mapping.** A top-level frame at a panel profile size, with role page or popup, is a GDL page or popup. A button is a component instance whose variants are its states (Normal, Active and Pressed). A text node is a label, a rectangle is a shape, and a frame with the slider role is a slider. Anything else follows the Appearance fidelity rule in the Brief.
- **Problems.** Constructs that GDL cannot express are reported in an Extron Problems panel, and export never drops them silently.
- **Branding.** The ArtCraft and FigCraft names, logos and app icon are trademarks and are replaced with GUI Studio branding. `LICENSE-MIT`, `LICENSE-APACHE` and the FigCraft copyright in `NOTICE` are kept, and GUI Studio's additions are listed in `NOTICE`.
- **No upstream contributions.** FigCraft's clean-room rule bars code written by an agent that operated a proprietary app it studies. Because GUI Designer was operated during this project, GUI Studio does not send its Extron work upstream.
- **Retired editor.** The eframe editor in `src/studio*` is retired once the fork can build the 1025M demo screen and export it.

## Consequences

- Positive:
  - Figma-class UX now, without re-implementing a canvas, panels, components, variables or undo.
  - Every Extron feature is a command, so it is scriptable through the CLI and MCP.
- Cost/risk:
  - The base is a large early-alpha codebase.
  - Builds depend on storytold git dependencies (the egui fork and craft-libs) and on wgpu.
  - Each upstream merge costs effort.
  - Most of FigCraft's features cannot be exported to GDL, so the Problems gate must stay strict to keep integrators' expectations honest.
- Revisit when: upstream stalls or changes its licence, the Extron identity mapping cannot preserve Control IDs through FigCraft's component sync, or merge cost outgrows the value of upstream updates.
