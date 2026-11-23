# Phase 03 — Components, variants and design tokens

## Goal

Reuse controls and styles the way Figma does while every instance remains an independent native GDL object with its own identity.

## In scope

- Components and instances with property overrides and reset. Create them with Ctrl+Alt+K, and edit them in an assets panel.
- Variants mapped onto Extron control states, such as Normal, Active and Pressed. Component properties cover text and visibility.
- Color, text and number styles and variables, with modes such as light and dark. Variables resolve on export.
- Copy and paste properties with Ctrl+Alt+C and Ctrl+Alt+V, choosing which properties to paste.
- ID manager:
  - Explicitly link a control to an existing Control ID, so that both share one GCP function.
  - Enforce GUI Designer's rules: same control type and same state count.
  - Auto-assign, offset and first-available allocation.
  - A searchable Where Used table that can filter duplicates.
- Assets panel for images, border styles, fonts and color variables. It can select and remove unused assets, and it can replace a font across the project.

## Acceptance criteria

- [ ] Editing a component updates its instances except overridden properties. Overrides show in the inspector and can be reset.
- [ ] Each instance exports with its own object key, Object ID and Control ID. Changing a component never changes an existing instance identity.
- [ ] Switching a variable mode updates the canvas, and the exported GDL contains resolved native values.
- [ ] Linking IDs across controls of a different type or a different state count is refused, and Problems names both controls. A linked pair exports with the same Control ID and keeps its own Object IDs. Changing the state count on one updates the other, while appearance stays independent, as GUI Designer does (study §13.2).
