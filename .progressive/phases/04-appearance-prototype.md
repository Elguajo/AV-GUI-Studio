# Phase 04 — Baked appearance, prototype preview and history

## Goal

Let designers use rich appearance that native properties cannot express, preview panel behavior, and recover earlier project versions.

## In scope

- Design-time gradients, shadows, strokes and images, following Appearance fidelity in the Brief. Buttons export them as per-state PNG images and passive decoration exports them as graphics.
- GDL states have two image layers, a background and a button or icon image, plus a per-state Blinking flag (study §13.3). EXUI/5 currently exposes one `image` per state. Using the icon layer needs an EXUI extension in the exporter.
- Prototype preview of page and popup navigation and control states. Runtime navigation in GCP remains out of scope.
- Project version history with named versions and restore.
- Quality aids from GUI Designer:
  - PNG snapshots of pages and of page and popup permutations;
  - a grayscale contrast check;
  - an estimate of panel memory use;
  - project metadata (job, PO, version and contacts).

## Acceptance criteria

- [ ] A button with baked appearance keeps its object key, Object ID, Control ID and state roles. Its GDL passes GUI Designer Verify and Save and Build.
- [ ] Problems warns about every baked object. A slider, label or value with appearance that native properties cannot express causes an export error at that object.
- [ ] Baked images render correctly on a physical panel. The result is recorded separately.
- [ ] Restoring a named version recreates the saved document, including all identities.
