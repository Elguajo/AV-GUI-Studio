# ADR-001 — Workspace canvas and visual language

Status: Superseded by [ADR-002](ADR-002-figcraft-base.md) for the editor implementation
Date: 2026-11-23

## Context

The Brief requires Figma interaction parity and a Figma UI3-style visual language for our version of Extron GUI Designer. The prototype already uses eframe/egui 0.33.3 with the `glow` backend, a scale-based canvas, and snapshot undo of `Project`. Figma's feel depends mainly on canvas behavior, such as crisp text at any zoom, cursor-anchored zoom, snapping and measurement, and on a dense, consistent panel system. egui's built-in `Scene` container scales tessellated output, so text blurs when zoomed (egui issue #4813). Documents are small: one 1280 × 800 screen with at most 64 elements and 1 MiB per project.

## Decision

- Keep eframe/egui for the whole workspace. Panels use egui widgets styled by GUI Studio tokens.
- The canvas is a custom viewport, not `egui::Scene`. Document geometry stays in panel pixels. A pure `Viewport` maps panel coordinates to screen coordinates using pan and zoom. Rendering happens in screen space with egui's `Painter`, and text is laid out at `font size × zoom` so it stays sharp. Hit testing, snapping and resize logic are pure functions and are unit-tested without the UI.
- Every document change goes through one edit path that records a `Project` snapshot for undo. Snapshots are cheap at the current document size. Selection, viewport and panel state are not part of undo history.
- Visual language uses semantic tokens in one module for surface, panel, border, text, muted text, accent and selection colors, and for spacing, radius and type scale. Light and dark themes are token sets. Inter is the UI font under the SIL OFL. Lucide is the icon set under the ISC license, bundled as an icon font with its license notice. Figma icons, logos and the exact Figma palette are not used.
- Selection colors carry Extron meaning. A blue accent marks the selection. A Control ID badge distinguishes addressable controls. Phase 03 adds distinct colors for component sources and instances.

## Consequences

- Positive: The work builds on the running prototype. The renderer requires no new graphics dependency. Interaction logic becomes testable. Panel and canvas share one theme.
- Cost/risk: Without anti-aliased compute rendering, preview fidelity for future gradients and shadows is limited. Text layout at many zoom levels can increase per-frame cost. Galleys must be cached per zoom step if profiling shows a cost.
- Revisit when: Phase 04 baked appearance needs a pixel-accurate preview, or profiling shows that the egui painter cannot hold 60 fps. The alternative is Vello on the eframe `wgpu` backend for the canvas only.
