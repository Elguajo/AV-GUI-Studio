# Phase 02 — Pages, popups, frames and layout

## Goal

Author and export real multi-page room interfaces with popups, frames, groups and Figma-style layout. Export them as absolute native GDL geometry.

## In scope

- Pages panel for GDL pages, popups, modal popups and the Offline Page. It covers the start page, popup groups (mutually exclusive popups), and popup references to a popup or a group. Hiding references lets the designer edit underneath them.
- Frames and groups with nested selection. Double-click or Enter selects children and Shift+Enter selects the parent.
- Design-time auto layout: Shift+A, direction, gap, padding, alignment, and hug, fill and fixed sizing.
- Constraints for frame resize.

## Acceptance criteria

- [ ] An integrator can manage multiple pages and popup instances in the editor and preview their interactions.
- [ ] Auto layout and constraints behave as in Figma for the supported options. Export emits only resolved absolute rects.
- [ ] A multi-screen EXUI/5 project produces a GDL that retains page and popup references and passes GUI Designer Verify and Save and Build.
- [ ] A style-only update of an existing control leaves its GCP action and feedback binding intact in an updated multi-page test project.
