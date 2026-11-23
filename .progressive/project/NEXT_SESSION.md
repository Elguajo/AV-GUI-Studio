# Next Session

Outcome: IN PROGRESS

## Current phase

Phase 01 — TLP Pro 1025M editor, Figma interaction core and GDL export

## Completed this session

- Clarified the Extron GUI Designer product goal and the roles of Figma Extron Exporter and Pencil.
- Implemented the Rust model, native desktop authoring prototype, EXUI export, and initial GDL build.
- Set the product direction to our version of Extron GUI Designer with Figma interaction parity and a Figma UI3-style visual language. Appearance that native properties cannot express is baked into PNGs. Recorded the workspace architecture in ADR-001 and planned Phases 02–04.

## Verification evidence

- Windows build, rustfmt, Clippy, three core tests, and the configured exporter integration test passed.
- Both versions of an export passed the independent verifier. Decoding confirmed Mute's object ID, control ID, key and press feedback state survived a style-only change.
- The Windows editor started and displayed its editable demo with the canvas, layers, and inspector.
- The EXUI/5 screen schema accepts a PNG `image` per button state. GUI Designer Build of baked images is unverified.
- 2026-11-23: the demo GDL passed GUI Designer Verify (0/0) and Save and Build. The round-trip changed no IDs, names or states. GUI Designer renumbered `zOrder` and dropped unused donor resources when allowed. GCP import is blocked at Import layout. See `docs/research/gui-designer-1.28.md` §13.
- 2026-11-23: direction changed to a FigCraft fork as the editor base (ADR-002). FigCraft b828e6c cloned to `../gui-studio-fork`.

## Current working state

RUNNABLE / GREEN

## Blockers / uncertainty

- The native editor currently supports one 1025M screen. The target layout still requires GUI Designer Build, configured GCP feedback and physical panel verification at their respective gates.

## Next action

Build and run the FigCraft-based fork in `../gui-studio-fork`, then add `crates/extron` (task-06-r1, ADR-002). The current eframe editor stays runnable until then.

## Active task prompt

.progressive/phases/01-1025m-editor.prompts.md#task-06-r1
