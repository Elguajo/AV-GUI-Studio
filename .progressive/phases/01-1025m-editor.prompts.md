# Phase 01 Task Records

## Task prompts

### task-01-r1

Task: Validate the TLP Pro 1025M exporter dependency.
Status: COMPLETED
Freshness: CURRENT
Source: Phase 01 task list and project bootstrap request.
Supersedes: NONE

#### Request

```text
Summary: Carry forward the product discussion from the linked GUI Designer chat and set up spec-driven development using Elguajo/Progressive-Context-Kit.
```

#### Prompt

```text
Use the existing exporter's tracked project-1025m EXUI/5 fixture and local 0.8.8 smoke-kit CLI to establish a baseline for the initial GUI Studio target. Package the fixture in a temporary directory, run validate-package and convert-package against the donor registry, then independently list controls in the GDL. Record the observed results in this task record. This check covers the exporter CLI only.
```

#### Freshness check

Resolved the exporter 0.8.8 smoke-kit CLI, fixture, and donor registry. The fixture and registry matched exporter commit `a0e3d3e04001bf0e6c91ce19586406d48e21deba` before execution.

#### Current checkpoint

The CLI baseline is complete.

#### Outcome / evidence

- 2026-11-23: `validate-package` accepted the tracked `tests/fixtures/exui-v5/project-1025m` EXUI/5 input for TLP Pro 1025M at 1280 × 800. The fixture contains 2 pages, 4 popups and 16 authored elements.
- `convert-package` generated a GDL and the independent verifier passed 230/230 checks. GDL SHA-256 for this run: `D9697C88968F73CFAF43B6BE2FAD19C9145CA15546F236782848B02F515811BC`.
- `list-controls` independently decoded the generated GDL, including the 1280 × 800 platform and Home's native `PBSlider` with Object ID 501 and Control ID 8101.
- The exporter report records GUI Designer and hardware verification as false for this run. No GCP verification was performed.

### task-02-r1

Task: Implement the first version of the Rust project model and EXUI/5 mapping.
Status: COMPLETED
Freshness: CURRENT
Source: Phase 01 task list.
Supersedes: NONE

#### Request

```text
Phase-derived task: Implement the data and export contracts for a single TLP Pro 1025M screen with source buttons, volume, Mute and labels.
```

#### Prompt

```text
After checking the roadmap, phase and live checkout, implement the versioned Rust document and deterministic EXUI/5 mapping for one TLP Pro 1025M screen. Cover source buttons, a volume slider, a Mute button with supported feedback states, and labels. Verify that save/reopen, reordering and style-only edits preserve identity, that invalid geometry and ID collisions produce diagnostics, and that the exporter's validate-package accepts the generated package. Record the commands and observed results in this task record.
```

#### Freshness check

The user requested a standalone Figma/Pencil-style Extron authoring tool. The 1025M model remains its first implementation target. Confirmed local Rust 1.100 and exporter EXUI/5 schemas and examples.

#### Current checkpoint

The versioned document and EXUI mapping are in src/project.rs and src/exui.rs.

#### Outcome / evidence

Three core tests passed. The 15-element 1025M demo passed validate-package and GDL conversion verified 182/182 checks.

### task-03-r1

Task: Deliver the native authoring prototype for one panel screen.
Status: COMPLETED
Freshness: CURRENT
Source: Current user request and Phase 01 canvas and initial export tasks.
Supersedes: NONE

#### Request

```text
И сделай свой инструмент наподобии Figma и pen.dev
```

#### Prompt

```text
Create a Windows Rust editor with an original visual workspace, a canvas, layer list and inspector. Connect the accepted project and EXUI model to interactive authoring, preview, undo, save/reopen and the initial export path. Verify the working binary and representative editor interactions.
```

#### Freshness check

The Rust document and exporter baseline were verified. Current user direction prioritizes native authoring. Cached eframe 0.33.3 provides the desktop window and canvas.

#### Current checkpoint

The desktop authoring prototype is implemented in src/studio.rs and src/studio/canvas.rs.

#### Outcome / evidence

- The Windows binary launched into the demo. Observed the canvas, zoom change, element selection and the inspector for Mute and Volume.
- cargo fmt --check, cargo test, Clippy with warnings denied, and the Windows build passed.
- The exporter integration test built the base and restyled GDL and independently read back the preserved Mute identity and changed appearance. GUI Designer Build, configured GCP and hardware were outside this test.

### task-04-r1

Task: Support a representative Pencil document as a panel authoring input.
Status: READY
Freshness: UNCHECKED
Source: Phase 01 Pencil import acceptance.
Supersedes: NONE

#### Request

```text
Phase-derived task: Support a documented subset of Pencil input while keeping GUI Studio native authoring independent.
```

#### Prompt

```text
After checking the phase, live checkout, and Pencil format, select one genuine supported .pen document. Implement its bounded, data-only mapping to the versioned GUI Studio model, including a reusable component instance and an override. Validate frame bounds, target control mapping, unsupported nodes and errors, and verify that the imported project can be edited, saved and exported.
```

#### Freshness check

Recheck the Pencil format version and locate a genuine fixture before defining the supported subset.

#### Current checkpoint

The native editor and initial export are runnable. Select a real Pencil input to define the supported import.

#### Outcome / evidence

Pending execution.

### task-05-r1

Task: Implement the Figma interaction core and the first visual language.
Status: SUPERSEDED
Freshness: UNCHECKED
Source: Current user request and the Phase 01 interaction checklist.
Supersedes: NONE

#### Request

```text
Мне как раз таки получить Figma experience во всем
Плюс хочется визуальный язык интерфейса нашей программы как у Figma
Но опять же не забываем что это наша версия Extron GUI Designer
```

#### Prompt

```text
After checking the phase, ADR-001 and the live checkout, bring the existing egui editor to the Phase 01 interaction checklist for the 1025M screen. Cover the viewport, selection, transforms, smart guides, layers, the inspector, undo and quick actions. Apply the ADR-001 visual language with tokens, light and dark themes, the Lucide icon font and Inter. Keep the project model and EXUI/5 identity unchanged. Do not add pages, frames, components or baked appearance. Unit-test the viewport, hit testing, snapping and resize logic. Then run the checklist on the Windows build and record each item.
```

#### Freshness check

Check src/studio.rs and src/studio/canvas.rs before execution. Snapshot undo, a scale-based canvas and an 8 px Shift snap existed when this record was created.

#### Current checkpoint

Not started. Superseded before execution.

#### Outcome / evidence

- 2026-11-23: a partial in-house implementation (geometry, viewport, canvas gestures, action table) was written and then removed when the user chose FigCraft as the base (ADR-002). Replaced by task-06-r1.

### task-06-r1

Task: Build GUI Studio on the FigCraft base and export the 1025M demo through it.
Status: COMPLETED
Freshness: CURRENT
Source: Current user request and ADR-002.
Supersedes: task-05-r1

#### Request

```text
Можешь полностью скопировать интерфейс Figma отсюда https://github.com/storytold/figcraft как раз он написан Rust
[Выбрано: Форкнуть FigCraft как основу]
Наш GUI Studio сделаем поверх FigCraft
```

#### Prompt

```text
In ../gui-studio-fork (a FigCraft clone with an upstream remote), first build and run the desktop app on Windows and record the result. Then:
- Rebrand the fork to GUI Studio: replace the ArtCraft and FigCraft names, logos and app icon, and keep LICENSE-MIT, LICENSE-APACHE and the FigCraft copyright in NOTICE.
- Add crates/extron with the panel profile, the control model, diagnostics and the EXUI/5 writer, ported from GUI-Studio src/project.rs and src/exui.rs.
- Store Extron identity in node plugin_data, following the exui-node/1 contract.
- Register extron.* commands for validate, export EXUI and build GDL.
- Rebuild the 1025M demo screen in the fork and export a GDL that passes the exporter's independent verifier with the same Object and Control IDs.
- Keep FigCraft's layering rules and checks.
Not in scope: multi-page, components as Extron states beyond buttons, GCP verification.
```

#### Freshness check

Recheck the fork commit, the toolchain (stable-x86_64-pc-windows-gnu 1.100) and FigCraft's own AGENTS.md before execution.

- 2026-11-23: Phase 01 is active and ADR-002 still governs this target. GUI-Studio is clean at 3b7e5b7. The clean local fork is at b828e6c7, with only the storytold upstream remote. Read the fork AGENTS.md and confirmed stable-x86_64-pc-windows-gnu, rustc and Cargo 1.100.0. The exporter 0.8.8 DLL, donor registry and .NET 10 runtime are present. Both Python commands resolve to Store aliases and failed with Python was not found, so the Progressive Python tools were not run. Local implementation and verification are authorized; publishing is outside this task.

#### Current checkpoint

Windows desktop and CLI builds and export acceptance are complete. The fork has an editable 1025M demo, GUI Studio branding, native metadata and all four registered Extron commands. The Windows menu and CLI GDLs match the original editor's demo. Final verification artifacts are in ../gui-studio-fork/target/task-06-evidence/.

#### Outcome / evidence

- 2026-11-23: `cargo build --release -p figcraft` failed while resolving git dependencies: "The server name or address could not be resolved". A retry with `CARGO_NET_GIT_FETCH_WITH_CLI=true` failed with "`git fetch` failed". `git ls-remote https://github.com/storytold/vectorcraft.git` failed with "curl 56 Recv failure: Connection was reset", while the GitHub API reports the repository as public. This is a network or transport problem on this machine, not a FigCraft code error. Nothing was compiled.
- 2026-11-23: Recovered the Git dependencies from shallow fetches of the locked revisions. The first isolated build reached crate downloads and failed on static.crates.io. The next reached compilation and failed on ring v0.17.14 because gcc.exe was absent. A local GCC 16.2.0 installation under the fork's ignored target directory then compiled a C probe with the matched MinGW headers and DLLs.
- 2026-11-23: cargo test --locked --offline --release -p gui-studio-extron --target-dir target/windows-fork passed two tests. They cover retained identity after an edit and node-level diagnostics for copied identity and rotated geometry.
- 2026-11-23: A probe calling the fork's Extron crate exported the demo. Exporter 0.8.8 validate-package accepted 1 page, 15 elements and 0 resources. convert-package's independent verifier passed 182/182 checks. list-controls decoded the GDL, and compare-graph reported identical payloads with zero differences. Both GDL files had SHA-256 857C516155A5A1152EA621BFB8AB1AA8AF391ADABF8FF6300BD6B5AA64B1056E. The reference was generated from the original editor's --demo-exui. The fork artifact is ../gui-studio-fork/target/task-06-evidence/ported-demo.gdl.
- 2026-11-23: Built the unmodified b828e6c7 snapshot on Windows with cargo build --offline --locked --release -p figcraft, using a workspace-local GCC and CRT import libraries. The first link failed on ktmw32, propsys, shlwapi and version. After supplying the verified MinGW CRT through LIBRARY_PATH, the build succeeded. Started figcraft.exe with a workspace-local profile. Windows reported a visible FigCraft window, and the localhost window.list endpoint reported the focused main window. Closed the baseline window after the check.
- Implementation commit in the local fork: `97bbf0e3c753371dd6d786fc653a1e3833144dae`.

##### Windows release and menu

- 2026-11-23: The Windows GNU release-profile build produced gui-studio.exe and gui-studio-cli.exe. The desktop window title and Windows ProductName, FileDescription and CompanyName were GUI Studio. The two upstream license files and the original attribution header in NOTICE were unchanged. The replacement ICO has nine sizes.
- Invoked New 1025M demo, Validate screen, Export EXUI and Build GDL through the desktop's File menu command path. Both saves completed in the native Windows dialogs. Validation returned 15 elements and zero diagnostics. Exporter 0.8.8 validated the UI EXUI and decoded the GDL; the UI GDL SHA-256 matched the original demo. Captured the application window and inspected its Windows UI Automation tree.

##### Headless round trip

- 2026-11-23: gui-studio-cli commands --filter extron listed all four commands. A CLI script ran the demo, validation, save, EXUI export and GDL build. Reopening the saved design and exporting again produced byte-identical EXUI. The independent verifier passed 182/182 checks and compare-graph found identical GDL payloads. GDL SHA-256 matched the original editor's demo, including page and all element Object IDs and all Control IDs.
- A second build to the existing GDL failed and preserved its hash. A root-confined CLI run refused external GDL build before creating an output.

##### Repository checks

- 2026-11-23: The fork's xtask layers and assets checks passed. xtask parity reported 215/215. Targeted Extron Clippy and the two crate tests passed. The full changed-package suite was started.
- 2026-11-24: The release-profile build passed for the Windows app, CLI and xtask. Repeated the native menu and CLI export on the final binaries. The final UI and CLI GDLs both retained SHA-256 857C516155A5A1152EA621BFB8AB1AA8AF391ADABF8FF6300BD6B5AA64B1056E with 182/182 checks. The final CLI save/reopen EXUI was byte-identical. Captured the window and read the accessibility tree, including the Aurora Meeting Room tab and Home and Mute layers. See ui-smoke-verified and cli-smoke-verified in the evidence directory.
- The Windows test run passed the editor (88 tests), CLI (27 unit and 35 integration tests), engine (2,277 tests), UI (1,352 tests), and the other touched crate suites. One stale collab-net brand assertion failed in the wide run and passed on the targeted re-run (four link tests passed). The final translation recheck passed all 28 tests. Final Clippy for all targets of the changed packages, rustfmt, xtask layers and assets passed. xtask parity reported 215/215. The test and build logs are under the fork's target directory.
- The build used the pinned Rust GNU toolchain, an isolated Cargo cache, and verified MinGW compiler and CRT files under the fork's target directory. The release profile used opt-level=1 without LTO for local checks. Four checksum-verified Inter faces from craft-fonts 26a7923 were provided through CRAFT_FONTS_DIR for the final build and test run. Without an embedded default face, the inherited board text-layout tests had failed.
- EXUI node assignments retain the exui-node/1 identity fields. Slider and shape records use the exporter's exui-node/2 extension. The Figma-compatible document codec and Cargo package IDs were retained for compatibility.
- Python was unavailable. The Progressive audit was not run. The scope of this verification is Windows, so xtask wasm was not run and only the Windows GNU target was installed.

### task-07-r1

Task: Display Extron validation findings in a Problems panel and select the affected layer.
Status: COMPLETED
Freshness: CHECKED (2026-11-25)
Source: Phase 01 Problems panel task and the diagnostics delivered by task-06-r1.

#### Request

```text
Replace single-error validation with object-level diagnostics in a Problems panel. Clicking a diagnostic selects its object.
```

#### Prompt

```text
In this repository (the editor moved here per ADR-003), connect the Extron Problems panel to the diagnostics returned by extron.validate. Use each diagnostic's nodeId to select and reveal its layer. Keep errors actionable in the CLI and keep invalid exports blocked. Use the Aurora Meeting Room demo and one invalid edit to verify that the panel updates and selection stays in sync with the canvas and Layers. Scope is the Problems panel for the current single-screen model.
```

#### Freshness check

Confirm task-06-r1 is completed, the editor retains the Extron command and node-level diagnostics, and the Phase 01 Problems panel is still open.

#### Current checkpoint

A Problems section under the layers lists every export problem of an Extron screen and follows edits and undo. `extron.validate` returns `nodeId`, `layer` and `message` per problem, and blocked exports fail with one line per problem.

#### Outcome / evidence

- 2026-11-25: Freshness confirmed. task-06-r1 is completed, `extron.validate` still returns node-level diagnostics, and the Phase 01 Problems task was open. The work copied from the former fork checkout was reviewed in this repository.
- Diagnostics now carry the layer name. Shared Control IDs follow GUI Designer's rules (study §13.2): a different control type or a button with a different state count gets GUI Designer's reason, and a compatible share still blocks because EXUI/5 needs unique Control IDs. Each message names the first owner of the key or ID. Off-screen geometry reports the 1280 × 800 bounds instead of a generic container error.
- The desktop Problems section re-validates only when the document snapshot changes. It reads the document directly, so validation never bumps the revision. Activating a problem selects its layer, reveals it in Layers and centres it, or zooms to fit when it is larger than the view. A page-level problem opens the page. Validate screen expands the section and toasts the problem count. Documents without Extron assignments show no section.
- `cargo test --locked --offline -p gui-studio-extron` passed 5 tests, including `a_layer_moved_off_the_screen_names_the_screen_bounds` and `shared_control_ids_follow_gui_designer_rules_and_name_the_first_owner`.
- `cargo test -p figcraft-ui-egui extron` passed the 3 `extron_problems_tests`. They cover the Aurora demo with an invalid edit (title moved to x = −10): the list updates without a revision bump, a click on the problem selects the title, Layers reveals it and the canvas centres it, undo empties the list, a blocked `extron.exportExui` names `"Room title" (node N)` and writes no file, and a non-Extron document shows no section. The 28 i18n tests passed.
- Clippy with warnings denied for all targets of `gui-studio-extron`, `figcraft-engine` and `figcraft-ui-egui` passed, as did rustfmt. The only warnings came from the workspace manifest, which this task did not change.
- Not run: the desktop window was not launched for a manual check, and the full package suites were not rerun.
- Scope: every current rule is an error. GUI Designer's one warning (an unreferenced popup) and the unique page and popup names need pages and popups, so the Warning severity and those rules moved to Phase 02. The panel already draws warnings differently.

### task-08-r1

Task: Author Extron controls from shared master components, so a master edit restyles every control made from it.
Status: COMPLETED
Freshness: CHECKED (2026-11-25)
Source: Phase 01 control-authoring task and the user's request for master components (2026-11-25).

#### Request

```text
Make master components of the Extron control entities in the editor, so that the design of buttons and their other parameters can be changed at any time by editing only the master element.
```

#### Prompt

```text
Extend the existing authoring (extron.insert, extron.insertBlock, extron.assign) so buttons and sliders are instances of one master per kind and kit, kept beside the panels on the canvas. A button's caption is an override on its instance and applies to all three states; appearance stays on the master's Normal, Active and Pressed variants. A slider's master holds its track, fill and thumb. Editing a master updates every instance on the canvas and in the next export, while each instance keeps its key, Object ID and Control ID. Report a problem in a master once, at the master layer. Labels and shapes stay plain layers; styles and variables for them are Phase 03. Keep the Aurora demo's EXUI and GDL hashes.
```

#### Freshness check

Commit 1c380be added touch panel projects, pages, UI kits and the insert, insertBlock and assign commands; each inserted button got its own variant set, and sliders were plain frames. No peer session was editing the working tree when this task started.

#### Current checkpoint

Buttons and sliders are instances of one master per kind and kit beside the pages. A master edit restyles every control made from it; captions, sizes and IDs stay with the instances.

#### Outcome / evidence

- 2026-11-25: `extron.insert` and `extron.insertBlock` place a button or slider as an instance of the kit's master (`guiStudio.master` = `button:<kit>` on a variant set, `slider:<kit>` on a component), made beside the pages the first time. A size other than the master's is a `size` override, and the caption and slider parts follow resize constraints. A caption is a `characters` override on the instance and replaces the caption of all three states in export. The extron crate takes the editor's component sync as a function (`SyncInstance`), because it cannot depend on the engine.
- Export: a button instance may override only its caption, size, position and name. A problem in a master is reported once, at the set or variant. Top-level components and variant sets are masters and never block export, so a master left without controls stays usable. Slider instances export from their resolved parts. Labels and shapes stay plain layers.
- `gui-studio-extron` (19 tests) passed, including `buttons_and_sliders_are_instances_of_one_master_that_restyles_them_all`, `a_problem_in_a_master_is_reported_once_at_the_master` and `a_button_refuses_a_look_changed_on_the_instance`. The engine test `editing_a_master_restyles_every_control_and_undo_brings_the_look_back` runs a real `node.set` on the master: every button follows on the canvas, captions stay, export stays valid, and undo restores the look. The full `figcraft-engine` (2283 + 51), `figcraft-ui-egui` (1357 + 12), `figcraft-mcp` and `gui-studio-extron` suites passed. Clippy with warnings denied and rustfmt passed for the three changed packages.
- The debug CLI kept the demo EXUI at `2B474E44…`. The kits script from `target/kits-evidence/` (two pages, 23 controls, two kits) produced a byte-identical EXUI `11959BBF…` with masters and a GDL with 294/294 checks. A new master-based project (three buttons, a slider and the Room audio block) built a GDL with 131/131 checks. Artifacts are in the local `target/masters-evidence/`.
- `Cargo.lock` gained `figcraft-layout` (resize constraints) and the dev-only `figcraft-engine` (component sync in tests) for `gui-studio-extron`.
- Not run: the desktop window was not launched, and no GDL was opened in GUI Designer.

### task-09-r1

Task: Extron section in the inspector and Control ID badges on the canvas.
Status: COMPLETED
Freshness: CHECKED (2026-11-26)
Source: Phase 01 acceptance criterion on the Extron-specific pieces the FigCraft base lacks, and the user's choice (2026-11-26) to work on editing convenience first.

#### Request

```text
Сперва Extron, потом аудит Figma.
```

#### Prompt

```text
Add the Extron pieces the FigCraft inspector lacks. With one Extron control selected, the Design panel shows an Extron section: the control type, Object ID (read-only), Control ID (editable, 1–65534, refused when another control uses it), a slider's minimum and maximum (editable, minimum below maximum), and for a button a Normal / Active / Pressed switch that shows that state's variant on the canvas without changing the export. A selected page shows its touch panel and resolution. Edits are one undo step each and go through an engine command, so the CLI and MCP get them too. Add a View toggle with one key that draws Control ID badges on the canvas: one per addressable control, at a fixed screen size, never over the control's caption. Keep the Aurora demo EXUI hash. The Figma interaction audit follows as a separate step.
```

#### Freshness check

task-08-r1 is committed in bcdd728. The inspector has no Extron section, and no Control ID badges exist. The only uncommitted change is the unrelated `apps/figcraft/src/window_frame.rs` fix.

#### Current checkpoint

The Extron section and the Control ID badges are in; the interaction audit is recorded below, and its gaps are an open Phase 01 task.

#### Outcome / evidence

- 2026-11-26: `gui_studio_extron::control_info` reads a layer's type, key, IDs, slider range, page profile and a button's state variants; `edit_control` changes the Control ID (1–65534, refused when another control has it; every page's ID counter moves past it, so a later control never gets it) or a slider's range (minimum below maximum). `extron.setControl {ids?, controlId?, min?, max?}` applies it as one undo step, a Control ID on one control at a time.
- The Design panel shows an Extron section for one selected Extron layer, right under the header: Object ID (read-only) and Control ID for a button or slider, Minimum and Maximum for a slider, a Normal / Active / Pressed switch for a button (it sets the instance's variant properties; the caption override stays and export writes every state), the Object ID of a label or shape, and the touch panel and resolution of a page.
- View ▸ Control ID badges (`view.controlIds`, Alt+I, kept in the preferences) draws each shown button's and slider's Control ID at a fixed screen size in its top-left corner, or above the control when the corner would cover a caption's ink or the control is too small. Alt+I is pinned in `shortcut_parity_tests` (59 pinned) and `docs/ui-parity.md`.
- Tests: `control_ids_and_slider_ranges_are_undoable_edits_refused_when_taken_or_out_of_range` and `a_button_shows_another_state_on_the_canvas_without_changing_its_export` (the EXUI package is byte-identical after the switch) in `figcraft-engine`; `extron_inspector_tests` (the Control ID typed in its field is one undo step, the Active switch shows the Active variant, a label has no Control ID field, a page shows `TLP Pro 1025M · 1280 × 800`, the badge toggle) and the badge placement unit test in `figcraft-ui-egui`. The full `gui-studio-extron` (19), `figcraft-engine` (2285 + 51), `figcraft-ui-egui` (1361 + 12) and `figcraft-mcp` (7 + 4 + 69) suites passed, as did Clippy with warnings denied for all targets of the three changed packages and `cargo fmt --check`. The debug CLI kept the demo EXUI at `2B474E44…` (`target/task09-evidence/`).
- Interaction audit against the Phase 01 checklist (code, keymap and existing tests; `docs/ui-parity.md` holds the measured Figma comparisons): present with tests are cursor-anchored zoom (2%–25,600%, wider than the checklist's 6400%), Ctrl+scroll and Ctrl+plus/minus, Space, middle-button and side-button pan, Shift+0/1/2, click, Shift+click and marquee, Esc, Ctrl+A, Tab, resize handles with Shift (ratio) and Alt (centre), 1/10 px nudges, Alt+drag and Ctrl+D, smart guides to edges and centres, Alt distances, Layers rename (double-click, Ctrl+R, F2), hide, lock, reorder by drag and Ctrl+[ / ], field scrubbing, arithmetic and mixed values, align and distribute, undo and redo, and the Actions palette on Ctrl+/. Gaps: no Match width, height or size; align acts on one layer's parent frame or on several layers' own bounds, never several layers to the panel; the palette lists every command and gives the reason only as an error after running one. Not checked: crisp text at high zoom, and none of it by hand in the window (the running app was an older build with an unsaved document in a minimised window, so it was left alone).

### task-10-r1

Task: Close the interaction audit's gaps: same width, height and size, and align and distribute to the frame.
Status: COMPLETED
Freshness: CHECKED (2026-11-26)
Source: The interaction audit in task-09-r1 and the user's go-ahead to continue (2026-11-26).

#### Request

```text
Потом сделаем коммит, пока идем дальше.
```

#### Prompt

```text
Cover GUI Designer's Layout menu (study §8) on top of the Figma align row. Add Same width, Same height and Same size, which size every selected layer like the first one selected, as one undo step. Add an Align to frame setting, kept like the snapping toggles: when it is on, the six align commands align each selected layer to its own frame (a touch panel page), and Distribute spaces the layers of one frame from its edges, so the outermost touch the frame. Commands take `to: selection|frame` to override the setting. Expose all of it in the Object menu, the Actions palette and the Design panel's alignment row. Showing an unavailable palette command and its reason before it runs needs a precondition mechanism the engine lacks; it is left as a decision for the user.
```

#### Freshness check

task-09-r1 is complete and uncommitted; the user asked to commit later. `align.*` aligns one layer to its parent frame and several to their combined bounds, and nothing matches sizes.

#### Current checkpoint

Same width, height and size and Align to frame are in. The Actions palette part waits for the user's decision.

#### Outcome / evidence

- 2026-11-26: `arrange.sameWidth`, `arrange.sameHeight` and `arrange.sameSize` `{ids?, reference?}` size every target but the reference (the first selected; the selection keeps click order) through the same property code as `node.set`, as one undo step; fewer than two layers is refused with the reason. An Extron button sized this way keeps a valid export (a `size` override).
- `view.alignToFrame` (Object ▸ Align to frame, kept in the preferences like the snapping toggles) makes the six `align.*` commands put each layer against its own frame, skipping layers on the page or in an auto layout flow, and makes `distribute.*` move the outermost layers of one frame to its edges and even the gaps (from two layers; layers of different frames are refused). `to: selection|frame` overrides the setting per call; `spacing` keeps the old distribution.
- The Design panel's alignment row always has a menu for layers (Alignment options): Align to frame with its check mark, Distribute, Tidy up, Same width, Same height and Same size, each greyed out with the reason when the selection cannot use it. The align buttons' disabled tip names Align to frame when it is on. Points keep their Distribute and tidy menu.
- Tests: `layout_menu_tests` (4, engine) and `layout_menu_ui_tests` (the menu turns on Align to frame, Same width sizes after the first selected, Align right puts both layers against the frame). `unknown_params_tests::ids_and_id_are_always_taken` now uses `arrange.front` as its `{ids?}` example, since the align commands take `to`. The full `figcraft-engine` (2289 + 51), `figcraft-ui-egui` (1362 + 12), `figcraft-mcp` (7 + 4 + 69) and `gui-studio-extron` (19) suites, Clippy with warnings denied and `cargo fmt --check` passed.
- Not done: the Actions palette still lists every command and gives an unavailable one's reason as an error after it runs. The engine has no way to ask whether a command can run without running it; that is a decision for the user. Not run: the desktop window, and the demo EXUI hash (no export code changed).

### task-11-r1

Task: Port the eframe prototype's identity tests to the master-based editor, with the GDL comparison as an opt-in test.
Status: COMPLETED
Freshness: CHECKED (2026-11-26)
Source: Phase 01 identity-test task and the user's choice (2026-11-26) to defer the palette's unavailable state and continue.

#### Request

```text
Ок отложи.
```

#### Prompt

```text
Port `save_reopen_and_restyle_retains_mute_identity` (tag `eframe-prototype`, `src/project.rs`) to the editor: in a TLP Pro 1025M project built from kit masters, restyling Mute's Active state through its master, reordering the layers and saving and reopening the file keep Mute's key, Object ID, Control ID and state roles, while the exported Active appearance changes. A copy of Mute (Duplicate) never inherits its identity: it is reported until assigned, and assigning gives new IDs. Deleting Mute and inserting a replacement gives new IDs. Port `tests/extron_export.rs` as an ignored test that builds two GDLs through the exporter and compares Mute's ObjectId, ControlId, Comments and PressFeedbackState through `list-controls`; run it here with the 0.8.8 smoke kit.
```

#### Freshness check

tasks 09 and 10 are complete and uncommitted. The extron crate's converter reads `GUI_STUDIO_EXTRON_CLI` and `GUI_STUDIO_EXTRON_DONORS`; the 0.8.8 smoke kit is at `../figma-extron-exporter/output/releases/0.8.8/smoke-kit`, and its CLI has `list-controls`.

#### Current checkpoint

Both prototype tests are ported; the GDL one passed against the 0.8.8 exporter.

#### Outcome / evidence

- 2026-11-26: `crates/engine/src/extron_identity_tests.rs`. `restyle_reorder_and_reopen_keep_mutes_identity_and_a_copy_or_replacement_never_inherits_it` builds a 1025M project from Aurora masters (HDMI, USB-C, Laptop, Mute, Volume, a title), restyles Mute's Active state on the master, sends Mute to the back and HDMI to the front, saves and reopens the `.figcraft`. Mute keeps its key, Object ID, Control ID and the Normal / Active / Pressed state names; Normal and Pressed keep their fills and Active exports `#3778A5`. Duplicate gives a copy that Problems reports ("Copied Extron assignment needs its own key and IDs.") until `extron.assign` gives it new IDs; deleting Mute and inserting a replacement gives IDs above the copy's, never Mute's.
- `a_style_only_change_keeps_mutes_gdl_identity` (ignored: needs dotnet, `GUI_STUDIO_EXTRON_CLI` and `GUI_STUDIO_EXTRON_DONORS`) builds two GDLs through the exporter before and after the master restyle and reads Mute back with `list-controls`: `Type` is `PBButton`, and `ObjectId`, `ControlId`, `Comments` and `PressFeedbackState` match while `States` differ. It passed with the 0.8.8 stage kit (`../figma-extron-exporter/output/releases/0.8.8/stage-4811fda9ba304219a951713b18c28f6f`, CLI and `samples/donors.json`); the 0.8.8 `smoke-kit` has no prepared donors (`donor-missing`, run `prepare-donor tlp-pro-1025m`).
- `gui_studio_extron` exports `Color` for the tests. The full `figcraft-engine` (2290 + 51, 4 ignored) and `gui-studio-extron` (19) suites, Clippy with warnings denied and `cargo fmt --check` passed.
- Not covered: the GCP half of the acceptance criterion (Mute's action and feedback after reopening the GCP project) still needs GCP, whose Import layout is blocked (study §13.5).

### task-12-r1

Task: Find the Extron exporter without setup and build GDL off the UI thread.
Status: COMPLETED
Freshness: CHECKED (2026-11-26)
Source: Phase 01 task "Restore exporter discovery and run GDL builds off the UI thread" and the user's choice (2026-11-26).

#### Request

```text
Сборка GDL в фоне
```

#### Prompt

```text
Find the exporter in this order: GUI_STUDIO_EXTRON_CLI with GUI_STUDIO_EXTRON_DONORS (the README's name; GUI_STUDIO_DONOR_REGISTRY, the prototype's, also works); the CLI and registry chosen in the app (kept in the preferences); a sibling figma-extron-exporter checkout with the 0.8.8 kit, taking the first registry whose donor files exist (the smoke kit has none prepared). When none is found, File ▸ Extron ▸ Build GDL… asks for Extron.Cli.dll and donors.json and keeps them; File ▸ Extron ▸ Choose GDL exporter… changes them. A missing donor for the project's panel is reported with the exporter's prepare-donor step. In the desktop app the build runs on a worker thread: a banner says the GDL is being built, the window stays responsive, Export EXUI and Build GDL are refused while it runs, and the result arrives as a toast. extron.buildGdl in the CLI and MCP keeps running in place and uses the same discovery.
```

#### Freshness check

task-11-r1 is complete; tasks 09–11 are uncommitted. `converter::build` reads only the two environment variables, and the app runs the build on the UI thread (about 12 s for the master project). The 0.8.8 `smoke-kit` has no prepared donors; `../figma-extron-exporter/samples/donors.json` and the 0.8.8 stage kit have them.

#### Current checkpoint

The exporter is found without setup, and the app builds GDL on a worker thread.

#### Outcome / evidence

- 2026-11-26: `gui_studio_extron::find_exporter` tries `GUI_STUDIO_EXTRON_CLI` with `GUI_STUDIO_EXTRON_DONORS` (or the prototype's `GUI_STUDIO_DONOR_REGISTRY`; a variable that is set but wrong is reported, not passed over), then the preferences (`Prefs::extron_exporter`), then a sibling `figma-extron-exporter` next to the program or the working folder or their parents: the 0.8.8 smoke kit's CLI with the first of the smoke kit's and the checkout's `samples/donors.json` whose donor files exist. `check_exporter` wants absolute files and at least one prepared donor; `build` now takes the exporter and first reports a missing donor for the project's panel with the `prepare-donor` step.
- Engine: `extron.buildGdl` uses the discovery (`cmd::extron::find_exporter`); `extron.exporter` reports the exporter and its source or why none was found; `extron.setExporter {cli, registry}` checks and keeps a choice in the preferences.
- App: Build GDL builds the project on a `gui-studio-gdl` thread; a canvas banner with a spinner says "Building <file>…", Export EXUI, Build GDL and Choose GDL exporter are greyed out in the menu and refused with a message while it runs, and `build_tick` turns the result into a toast. Without an exporter, Build GDL asks for `Extron.Cli.dll` and then `donors.json`; File ▸ Extron ▸ Choose GDL exporter… (`extron.chooseExporter`) does the same alone. The README describes the order.
- Tests: `converter::tests` (2: sibling discovery with the prepared registry, missing donors and relative paths, and the not-found message), `the_chosen_exporter_is_checked_kept_in_the_preferences_and_reported` (engine), and `gdl_build_tests` (the build runs on its thread, Export EXUI waits, a failing exporter ends in a "GDL build failed." toast and no file). The ignored `a_style_only_change_keeps_mutes_gdl_identity` now finds the exporter as the app does; run with `--include-ignored` and no variables set, it found the sibling checkout here and built both GDLs.
- The full `figcraft-engine` (2291 + 51), `gui-studio-extron` (21), `figcraft-mcp` and `figcraft-ui-egui` (1363 of 1364 + 12) suites, Clippy with warnings denied and `cargo fmt --check` passed. The one failure, `extron_problems_tests::the_layer_list_stops_above_the_problems_section`, is another session's uncommitted test for its own Problems-section layout change in the same checkout, not part of this task.
- Not run: the build was not tried by hand in the desktop window, and the file pickers of Choose GDL exporter were not driven in a test.
