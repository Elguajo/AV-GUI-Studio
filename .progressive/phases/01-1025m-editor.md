# Phase 01 — TLP Pro 1025M editor, Figma interaction core and GDL export

## Goal

Take one meeting room screen through import or authoring, visual editing, persistence, GDL export, and a style-only revision in a configured GCP project. Make authoring feel like Figma for every interaction this screen needs.

## Context hints

- [ADR-002 — GUI Studio on top of FigCraft](../decisions/ADR-002-figcraft-base.md). It supersedes the editor part of ADR-001. The local fork is in `../gui-studio-fork`.
- [GUI Designer 1.28 functional study](../../docs/research/gui-designer-1.28.md). Its parity map assigns GUI Designer capabilities to phases.

## Tasks

- [x] Validate the EXUI/5 → GDL dependency with the exporter's tracked TLP Pro 1025M fixture. See [task-01-r1](01-1025m-editor.prompts.md#task-01-r1).
- [x] Implement a versioned Rust project model and deterministic EXUI/5 mapping for the first screen.
- [x] Connect Windows canvas authoring, layers, inspector, states, preview, and save/reopen to the model. See [task-03-r1](01-1025m-editor.prompts.md#task-03-r1).
- [ ] Select a genuine, versioned `.pen` fixture with reusable controls and supported overrides. Implement and test that mapping, with object-level diagnostics for unsupported nodes.
- [x] Connect initial EXUI and GDL exports to the existing converter.
- [ ] ~~Implement the Figma interaction core according to ADR-001 ([task-05-r1](01-1025m-editor.prompts.md#task-05-r1)).~~ Superseded by ADR-002.
- [ ] Build GUI Studio on FigCraft: rebrand the fork, add `crates/extron` with the ported model and EXUI/5 writer, add `extron.*` commands, and keep Extron metadata in `plugin_data`. Author the 1025M demo and export a GDL that passes the independent verifier. See [task-06-r1](01-1025m-editor.prompts.md#task-06-r1).
- [ ] Replace single-error validation with object-level diagnostics in a Problems panel. Clicking a diagnostic selects its object. Mirror GUI Designer's rules from the study (§13.2):
  - shared Control IDs require the same control type and state count;
  - page and popup names are unique;
  - an unreferenced popup is a warning;
  - geometry stays inside the panel.
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
- [ ] Check the interaction checklist against the FigCraft base on Windows, and add only the Extron-specific pieces it lacks, such as Control ID badges and panel-profile frames.
- [ ] Invalid geometry, duplicate IDs, malformed files, and unsupported input report an actionable object or file. Failed exports leave existing project files and GDL outputs intact.
- [ ] An independently read GDL confirms native types and IDs. Both copies pass Verify and Save and Build in GUI Designer.
- [ ] A style-only change to Mute preserves object key, Object ID, Control ID, and state identity. The revised layout preserves Mute's action and visual feedback when the GCP project is reopened. A removed or incompatible replacement never silently inherits those bindings.

## Verification

- Test persistence and EXUI/5 mapping for deterministic IDs, style edits and reordering, invalid geometry, and conflicting IDs. Include tests for import failure and the supported component override.
- Compare the canvas render of the demo Home page with GUI Designer's reference snapshot `docs/research/assets/gd-1.28-aurora-home-1025m.png`, and record the tolerance used.
- Unit-test the Extron mapping from FigCraft nodes to EXUI/5 without the UI. Run the interaction checklist manually on the Windows build of the fork and record the result for each item.
- Use `validate-package`, `convert-package`, and `list-controls` with the chosen CLI. Keep verifier output for both export versions and the update report. In GUI Designer build both copies, then reimport the revised layout in GCP and reopen the configured test project. Record each result separately.
