# Architecture — GUI Studio

## Current state and target

The Windows desktop editor uses eframe/egui 0.33.3 for the authoring workspace and a versioned local JSON document for one TLP Pro 1025M screen. The editor emits EXUI/5 and can invoke the existing `figma-extron-exporter` CLI to build and independently verify a new GDL. Selective updating of configured layouts and the Pencil source adapter are planned.

Target per [ADR-002](../decisions/ADR-002-figcraft-base.md): GUI Studio becomes a downstream fork of FigCraft. The local clone is `../gui-studio-fork`, with an `upstream` remote, based on FigCraft `b828e6c` (2026-11-23).
- FigCraft supplies the engine, document model, command registry and UI3-style egui workspace.
- GUI Studio adds the `crates/extron` crate and `extron.*` commands, with Extron metadata in node `plugin_data`.
- The EXUI/5 writer is ported from `src/project.rs` and `src/exui.rs`.
- The current eframe editor in `src/studio*` stays runnable until the fork exports the 1025M demo, then it is retired.

```text
Supported .pen import ──> GUI Studio project model <──> file, canvas, inspector, preview
                                      │
                                      └──> EXUI/5 ──> exporter CLI ──> editable GDL
                                                                          │
                                                    Extron GUI Designer Verify and Build
                                                                          │
                                                             GCP actions and feedback
```

## Reuse from Figma Extron Exporter

- Use EXUI/5 profiles, typed controls, state roles, persistent keys and IDs, object-level validation, independent GDL inspection, and selective layout updates. Use Extron Kit and exporter fixtures as references for control semantics and verification.
- GUI Studio owns native authoring and its project model. Figma and Pencil documents can supply input through adapters.

[Figma Extron Exporter](https://github.com/Elguajo/figma-extron-exporter)

## Contracts

- The local Rust document is the source of truth. Store a format version, target profile, element keys, geometry, properties, and state definitions. Persist assigned Object IDs and Control IDs across save, reopen, edits, and export. Allocate new IDs only for new identities. Match supported imported nodes to stable source keys and report unresolved or ambiguous mappings.
- Feed preview and export from the same normalized document. The EXUI/5 adapter emits native types, geometry, state roles, and significant IDs. Keep the exporter CLI path and supported version explicit. Publish a new output only after the CLI reports successful validation and conversion.
- Design-time constructs such as auto layout, variables and components resolve to native values and absolute rects in the EXUI/5 adapter. Baked appearance follows the Brief's Appearance fidelity rule. Buttons emit PNGs through the `image` field of EXUI/5 states, and passive decoration emits `graphic` (PBImage). Sliders, labels and values are never baked. Every instance keeps its own key, Object ID and Control ID.
- Treat `.pen` as a versioned input. Use a bounded mapping from a real supported document, including reusable references and overrides used by the fixture. Identify unsupported versions, features, and ambiguous metadata. Accept data only and handle references to external files or executable content as unsupported in the initial importer.
- For configured-layout updates, use the exporter's plan and selection workflow on copies. Surface identity, type, and state conflicts before producing an updated layout. Preserve GCP bindings only for the supported and verified cases.

## Prior evidence and limits

- The Rust feasibility probe mapped an MCP snapshot of Extron Kit for TLP Pro 300M into EXUI/5. The converter produced nine native elements and the independent verifier reported 129/129 checks. The snapshot demonstrated reference and override resolution for that fixture. Direct `.pen` import, visual fidelity, GUI Designer build, and panel operation were outside that run. The probe and its report are local under `../figma-extron-exporter/output/pen-rust-probe/`. [Input snapshot](https://github.com/Elguajo/figma-extron-exporter/blob/a0e3d3e04001bf0e6c91ce19586406d48e21deba/docs/design/extron-kit/apple-light-2026-10-24.snapshot.json)
- A separate TLP Pro 1025M update preserved the Music button's action and visual feedback in a reopened GCP project. That protocol covers one GCP project and fixture pair. Physical execution and the new GUI Studio scenario remain open. [GCP validation record](https://github.com/Elguajo/figma-extron-exporter/blob/a0e3d3e04001bf0e6c91ce19586406d48e21deba/docs/verification/results/2026-10-23-gcp-layout-update.md)
- [EXUI/5 contract](https://github.com/Elguajo/figma-extron-exporter/blob/a0e3d3e04001bf0e6c91ce19586406d48e21deba/docs/exui-v5.md) · [Configured layouts](https://github.com/Elguajo/figma-extron-exporter/blob/a0e3d3e04001bf0e6c91ce19586406d48e21deba/docs/updating-configured-projects.md) · [GUI Designer and GCP boundary](https://github.com/Elguajo/figma-extron-exporter/blob/a0e3d3e04001bf0e6c91ce19586406d48e21deba/docs/gui-designer-gcp-boundary.md)
- [GUI Designer 1.28 functional study](../../docs/research/gui-designer-1.28.md) is a black-box study from the bundled help and an unsaved Mach 1025M project. It covers ID sharing rules, states, resources, layout tools and Verify behavior, with a phase parity map.
- [Pencil file format](https://docs.pencil.dev/for-developers/the-pen-format) · [Pencil editing workflow](https://docs.pencil.dev/core-concepts/pencil-interface)
