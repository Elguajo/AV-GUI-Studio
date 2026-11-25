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
