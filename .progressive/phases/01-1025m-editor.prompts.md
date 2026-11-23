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
Status: READY
Freshness: UNCHECKED
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

#### Current checkpoint

Clone created at FigCraft b828e6c. Release build of `figcraft` started.

#### Outcome / evidence

Pending execution.
