# Phase 01 — TLP Pro 1025M editor, Figma interaction core and GDL export

## Goal

Take one meeting room screen through import or authoring, visual editing, persistence, GDL export, and a style-only revision in a configured GCP project. Make authoring feel like Figma for every interaction this screen needs.

## Context hints

- [ADR-002 — GUI Studio on top of FigCraft](../decisions/ADR-002-figcraft-base.md). It supersedes the editor part of ADR-001. [ADR-003](../decisions/ADR-003-independent-repository.md) moved the editor into this repository. The eframe prototype referenced below is at tag `eframe-prototype`.
- [GUI Designer 1.28 functional study](../../docs/research/gui-designer-1.28.md). Its parity map assigns GUI Designer capabilities to phases.

## Tasks

- [x] Validate the EXUI/5 → GDL dependency with the exporter's tracked TLP Pro 1025M fixture. See [task-01-r1](01-1025m-editor.prompts.md#task-01-r1).
- [x] Implement a versioned Rust project model and deterministic EXUI/5 mapping for the first screen.
- [x] Connect Windows canvas authoring, layers, inspector, states, preview, and save/reopen to the model. See [task-03-r1](01-1025m-editor.prompts.md#task-03-r1).
- [ ] Select a genuine, versioned `.pen` fixture with reusable controls and supported overrides. Implement and test that mapping, with object-level diagnostics for unsupported nodes.
- [x] Connect initial EXUI and GDL exports to the existing converter.
- [ ] ~~Implement the Figma interaction core according to ADR-001 ([task-05-r1](01-1025m-editor.prompts.md#task-05-r1)).~~ Superseded by ADR-002.
- [x] Build GUI Studio on FigCraft: rebrand the fork, add `crates/extron` with the ported model and EXUI/5 writer, add `extron.*` commands, and keep Extron metadata in `plugin_data`. Author the 1025M demo and export a GDL that passes the independent verifier. See [task-06-r1](01-1025m-editor.prompts.md#task-06-r1).
- [x] Fix text size units. Extron font sizes are points, and the exporter converts px to pt by ×0.75 (`figma-extron-exporter/src/figma-plugin/NATIVE_TEXT.md`). The fork stored the pt value directly as `font_size` in px, so the canvas drew text 25 % smaller than the panel. The demo now stores pt × 4/3 px, and the adapter exports whole points (`PX_PER_PT` in `crates/extron/src/adapter.rs`).
  - 2026-11-24 evidence: the regression test `canvas_text_is_px_and_exports_as_points` and the four other `gui-studio-extron` tests passed. Clippy with warnings denied and rustfmt passed for the crate. The rebuilt CLI ran demo, validate, save, EXUI export and GDL build. The EXUI SHA-256 `2B474E44…D06207F` matched the task-06 baseline and the reopened export. The GDL SHA-256 `857C5161…4B1056E` matched the original demo, and the verifier passed 182/182 checks. Artifacts are in the local `target/font-units-evidence/` of the former fork checkout.
- [x] Move the editor into this repository as an independent product ([ADR-003](../decisions/ADR-003-independent-repository.md)).
  - 2026-11-24 evidence: every imported blob matched fork commit `97bbf0e3` by hash, apart from the files ADR-003 moves or leaves out. The release build of `gui-studio` and `gui-studio-cli` from this repository passed, as did the three committed `gui-studio-extron` tests and `xtask layers` and `assets`. The CLI demo round trip produced the baseline EXUI `2B474E44…`, also after reopening, and the GDL `857C5161…` with 182/182 checks. Artifacts are in the local `target/import-evidence/`. The desktop window was not launched from this build.
- [ ] Author Extron controls in the editor. Add commands for an empty 1025M panel and for inserting or assigning Button, Slider, Label and Shape, using the presets and default sizes from `Kind::create` and `Kind::default_size` in the eframe `src/project.rs`. Allocate Object and Control IDs from a monotonic counter stored in the panel frame's `plugin_data`, so a deleted ID is never handed to a new control.
- [x] Change request (2026-11-25): choose the touch panel when a project starts, as in GUI Designer, and add pages with an Illustrator-style Artboard tool. Pages pull the page part of Phase 02 forward; popups, navigation and choosing the start page stay there.
  - The panel catalog (`PROFILES` in `crates/extron/src/model.rs`) holds the exporter's two EXUI/5 profiles: TLP Pro 1025M (1280 × 800) and TLP Pro 300M portrait (320 × 480), from figma-extron-exporter `docs/extron-format.md`. `extron.newPanel {profile}` and `extron.profiles` expose it, and the New touch panel project dialog picks it.
  - Every panel-sized top-level frame with a Page assignment is a page. The start page is the one with the lowest Object ID. One page keeps the single-screen EXUI/5 form, so existing hashes hold. More pages use the project form, with the other pages `shownBy: "program"`. Object and Control IDs come from one counter kept on every page.
  - The Artboard tool (`tool.artboard`, Shift+O) and `extron.addPage` add a page with the start page's panel, project name and background.
  - 2026-11-25 evidence: 12 `gui-studio-extron` tests, the full `figcraft-engine` (2281 + 51), `figcraft-mcp` and `figcraft-ui-egui` (1356) suites passed. Clippy with warnings denied and rustfmt passed for the three changed packages. The debug CLI rebuilt the demo with the baseline EXUI `2B474E44…` and GDL `857C5161…` (182/182). A two-page 1025M project built a GDL with 84/84 checks, and a two-page 300M project one with 59/59. Artifacts are in the local `target/artboard-evidence/`. Neither multi-page GDL was opened in GUI Designer, and the desktop window was not launched.
- [x] Change request (2026-11-25): UI kits, built in and installed from files. See [UI kits](../../docs/ui-kits.md).
  - A kit (`gui-studio-kit/1`, `crates/extron/src/kit.rs`) holds the colours of native button states, sliders, labels and shapes, the page background, and blocks of controls. It holds only what a native control keeps in a GDL. Aurora reproduces the earlier presets and stays the default; Daylight and Graphite are the other built-in kits.
  - A project keeps its kit as JSON in every page's `plugin_data` (`guiStudio.kit`). Installed kits are files in the app's `UI Kits` folder, beside `Plugins`. The CLI keeps them for the run.
  - Commands: `extron.kits`, `installKit`, `removeKit`, `useKit` and `insertBlock`, plus `kit` on `newPanel` and `insert`. The New touch panel project dialog picks the kit, and File → Extron → UI kits… inserts controls and blocks, installs and removes kits and switches the project's kit.
  - 2026-11-25 evidence: 16 `gui-studio-extron` tests, the full `figcraft-engine` (2282 + 51), `figcraft-mcp` and `figcraft-ui-egui` (1357 + 12) suites passed. Clippy with warnings denied and rustfmt passed. The debug CLI kept the demo EXUI at `2B474E44…`. A two-page Graphite 1025M project with every 1025M block and a Daylight block (23 controls) built a GDL with 294/294 checks, and a Daylight 300M project with the Power and Volume blocks one with 88/88. A block too large for the 300M was refused. Artifacts are in the local `target/kits-evidence/`. The GDLs were not opened in GUI Designer, and the desktop window was not launched.
- [ ] Port the identity tests from the eframe prototype: restyle, reorder and reopen keep Mute's key and IDs, and a replacement after delete gets a new ID (`save_reopen_and_restyle_retains_mute_identity`). Port the ignored GDL test `tests/extron_export.rs`, which builds two GDL versions and compares `ObjectId`, `ControlId`, `Comments` and `PressFeedbackState` through `list-controls`.
- [ ] Restore exporter discovery and run GDL builds off the UI thread:
  - read `GUI_STUDIO_EXTRON_CLI` and `GUI_STUDIO_DONOR_REGISTRY`, the name the README and the GDL test use (the fork currently reads `GUI_STUDIO_EXTRON_DONORS`);
  - fall back to a sibling `figma-extron-exporter` 0.8.8 smoke kit, then to a dialog that selects the CLI DLL and the registry;
  - run `extron.buildGdl` in a worker thread with a progress status and disabled export actions, following the `figcraft-save` thread in `crates/ui-egui/src/app/files.rs`.
- [x] Replace single-error validation with object-level diagnostics in a Problems panel. Clicking a diagnostic selects its object. See [task-07-r1](01-1025m-editor.prompts.md#task-07-r1). Mirror GUI Designer's rules from the study (§13.2) for the single-screen model:
  - shared Control IDs require the same control type and state count;
  - geometry stays inside the panel.
  - 2026-11-25 evidence: 5 `gui-studio-extron` tests, 3 `extron_problems_tests` and the 28 i18n tests passed. Clippy with warnings denied and rustfmt passed. Details are in task-07-r1. The Warning severity, the unreferenced-popup warning and unique page and popup names moved to Phase 02, because they need pages and popups.
- [ ] Ask for one exporter fix in `figma-extron-exporter`: write `zOrder` equal to the list index. GUI Designer renumbers z-order on save (study §13.1). Donor resources are carried over deliberately and can be removed in Save and Build; the exporter documents this in `docs/extron-format.md`.
- [ ] Connect configured-layout revision to the exporter plan and selection workflow.
- [ ] Verify the base and revised GDL in GUI Designer and the bindings in GCP. Record physical-panel verification separately when hardware is available.
  - 2026-11-23, partial evidence: the demo GDL (`857C5161…`) passed Verify with 0 errors and 0 warnings, and Save and Build succeeded in GUI Designer 1.28.0.7. GCP import of the built layout is still blocked at Import layout (study §13.5).

## Task prompts

[Phase 01 task records](01-1025m-editor.prompts.md)

## Acceptance criteria

- [ ] A user can create and edit a screen with source buttons, a volume slider, Mute feedback states, and labels. Saving and reopening preserves geometry, appearance, state roles, and IDs.
- [ ] The supported `.pen` fixture opens as editable native controls. The preview shows the resulting types, geometry, and states in the target panel bounds.
- [ ] Interaction checklist. Each item behaves as it does in Figma:
  - Zoom anchored to the cursor from 2% to 6400%. Ctrl+scroll and Ctrl+plus/minus zoom; Space+drag and middle-drag pan. Shift+0 zooms to 100%, Shift+1 fits and Shift+2 zooms to the selection. Text stays crisp at every zoom.
  - Click, Shift+click and marquee select. Esc clears the selection. Ctrl+A selects all and Tab selects the next sibling.
  - Edge and corner handles resize. Shift locks the aspect ratio and Alt resizes from the center. Arrows nudge by 1 px and Shift+arrows by 10 px. Alt+drag and Ctrl+D duplicate.
  - Smart guides snap to sibling and panel edges and centers. Holding Alt shows red distance measurements.
  - Layers panel: rename by double-click or Ctrl+R, hide with Ctrl+Shift+H, lock with Ctrl+Shift+L, reorder by drag or Ctrl+[ and Ctrl+]. Selection stays in sync with the canvas.
  - Inspector numeric fields support drag scrubbing on the label, arithmetic such as `100/2`, and mixed values for multi-selection.
  - Align and distribute the selection to each other or to the panel. Match width, height or size to the first selected object. These cover the GUI Designer Layout menu.
  - One key toggles Control ID badges, which never cover the object's text.
  - Ctrl+Z and Ctrl+Shift+Z undo and redo every document change. Ctrl+/ opens quick actions, which show why an action is unavailable.
- [ ] The workspace uses FigCraft's theme tokens and code-drawn icons in light and dark themes, branded as GUI Studio.
- [ ] Check the interaction checklist against the FigCraft base on Windows, and add only the Extron-specific pieces it lacks, such as Control ID badges, panel-profile frames, and an Extron inspector section with Object ID, Control ID, a Normal / Active / Pressed state switch and the panel profile, as the eframe inspector had.
- [ ] Invalid geometry, duplicate IDs, malformed files, and unsupported input report an actionable object or file. Failed exports leave existing project files and GDL outputs intact.
- [ ] An independently read GDL confirms native types and IDs. Both copies pass Verify and Save and Build in GUI Designer.
- [ ] A style-only change to Mute preserves object key, Object ID, Control ID, and state identity. The revised layout preserves Mute's action and visual feedback when the GCP project is reopened. A removed or incompatible replacement never silently inherits those bindings.

## Verification

- Test persistence and EXUI/5 mapping for deterministic IDs, style edits and reordering, invalid geometry, and conflicting IDs. Include tests for import failure and the supported component override.
- Compare the canvas render of the demo Home page with GUI Designer's reference snapshot `docs/research/assets/gd-1.28-aurora-home-1025m.png`, and record the tolerance used.
- Unit-test the Extron mapping from FigCraft nodes to EXUI/5 without the UI. Run the interaction checklist manually on the Windows build of the fork and record the result for each item.
- Use `validate-package`, `convert-package`, and `list-controls` with the chosen CLI. Keep verifier output for both export versions and the update report. In GUI Designer build both copies, then reimport the revised layout in GCP and reopen the configured test project. Record each result separately.
