# Project Brief — GUI Studio

## Outcome

GUI Studio is our version of Extron GUI Designer: a standalone Rust desktop application for authoring and maintaining TouchLink Pro interfaces. Extron defines what the product can author: panel profiles, pages, popups, native controls, states, Control IDs and GDL. Figma defines how authoring feels. Designers create interfaces directly in GUI Studio. Integrators receive editable GDL and can revise a configured interface while keeping existing control bindings.

[Extron GUI Designer capabilities](https://www.extron.com/product/software/guidesigner)

## Users and jobs

- An AV designer composes a room interface from native controls, edits it on a canvas, and previews states at the target panel size.
- An AV integrator validates and exports the layout, then assigns device actions and feedback in Extron Global Configurator Professional (GCP).

## Product scope

- Support a complete panel-authoring workflow, including panel profiles, pages, popups, native controls and states, reusable design resources, layout validation and GDL export.
- Prioritize direct creation on the canvas. The Figma Extron Exporter is the source of tested export behavior and practical authoring lessons. Pencil provides an optional import format and secondary workflow references.

## Experience principle

- Interaction parity with Figma. A Figma user can work in GUI Studio from muscle memory: canvas navigation, selection, transforms, smart guides, layers, the property inspector, shortcuts, frames and auto layout, components, variants, styles and variables. The roadmap delivers parity in phases.
- Parity is limited to Extron authoring. Every Figma-style feature maps onto an Extron construct, such as variants onto control states and pages onto GDL pages. Design-time features such as auto layout and variables resolve to native values on export.
- Visual language. The workspace follows Figma UI3 conventions, including compact density, neutral panels, layers on the left, the inspector on the right, a floating bottom toolbar, small type, a blue selection accent, and light and dark themes. GUI Studio uses its own design tokens, an open-licensed icon set and its own branding. It does not copy Figma's icons, logos or exact palette.

## Appearance fidelity

- Native first. Appearance that a native GDL property can express is exported natively.
- A designer may use appearance that native properties cannot express. Export then bakes that appearance into a PNG and keeps the native object and its identity, and Problems lists a warning for the affected object. Buttons use per-state images. A button caption is never baked into these images by default and stays native state Text, aligned middle-center or bottom-center. Text Feedback and per-state caption color then keep working, and no separate overlay Label is created. Baking the whole state, caption included, is an explicit per-button choice. Passive decoration becomes a graphic. A slider, label or value is not rasterized. Unsupported appearance on these controls is an export error at the object.

## First release

- First scenario: a single TLP Pro 1025M meeting room screen with source selection, volume, Mute, and labels.
- Create and edit a local project, save and reopen it, preview control states, and build a GDL.
- Provide the core Figma interaction set and the first visual language. Phase 01 lists the specific interactions.
- Import one genuine `.pen` fixture using a documented subset of reusable components and instance overrides.
- Diagnose export issues at the relevant object. Support a style-only update to a configured layout with the same control identity.

## Explicit constraints

- Rust for the editor, built as an independent product on a snapshot of the open-source FigCraft editor (MIT OR Apache-2.0, [ADR-002](../decisions/ADR-002-figcraft-base.md), [ADR-003](../decisions/ADR-003-independent-repository.md)). FigCraft and ArtCraft branding are not used. Extron is the first export target.
- Preserve editable native controls and their significant identifiers in GDL. GCP owns device commands, feedback, and program navigation.
- Work is local. Real-time collaboration, AI generation, vector networks, rotation and additional export targets are out of scope until the brief changes.

## Material assumptions

- Windows is the initial target because the existing export and Extron verification workflow run there.
- TLP Pro 1025M is the first acceptance target. The TLP Pro 300M probe provides supporting evidence for the architecture.
- Baked state images are accepted by EXUI/5. GUI Designer Build and panel rendering of baked images are unverified.

## Ubiquitous Language

- Control ID — Extron identifier used to address an interactive control or feedback target in GCP.
- Object ID — identifier used for the GDL object and its structure.
- EXUI/5 — the package contract consumed by the existing Extron exporter.
- GDL — editable layout opened and built in Extron GUI Designer.
- Interaction parity — behavior that matches Figma for the same user action.
- Baked appearance — a PNG generated at export from design-time appearance that native properties cannot express. Its native object keeps its identity.

## Success criteria

An integrator can import or author the target screen, save and reopen it, export a GDL that passes Extron GUI Designer Verify and Save and Build, and repeat the export after editing Mute's appearance. The new layout retains the same Control ID and GCP still shows the configured action and visual feedback after reopening. Physical-panel operation needs its own recorded result. A Figma user completes the Phase 01 interaction checklist without instruction.

## Classification

- Planning depth: FULL for the file and GCP compatibility boundary and the workspace architecture.
- Complexity: XL
- Risk: Medium
