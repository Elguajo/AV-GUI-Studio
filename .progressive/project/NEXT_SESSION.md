# Next Session

Outcome: IN PROGRESS

## Current phase

Phase 01 — TLP Pro 1025M editor, Figma interaction core and GDL export

## Completed this session

- Completed [task-09-r1](../phases/01-1025m-editor.prompts.md#task-09-r1): the Design panel's Extron section (Object ID, editable Control ID, slider range, the Normal / Active / Pressed switch for the state the canvas shows, a page's touch panel) through `extron.setControl`, and View ▸ Control ID badges (Alt+I). Its interaction audit found three gaps.
- Completed [task-10-r1](../phases/01-1025m-editor.prompts.md#task-10-r1): Same width, Same height and Same size after the first selected layer, and Align to frame for align and distribute.
- The palette's unavailable state is deferred by the user, to be done later together with greying out the main menus.
- Completed [task-11-r1](../phases/01-1025m-editor.prompts.md#task-11-r1): Mute's identity survives a master restyle, reordering and save/reopen; a copy or a replacement never inherits it; the opt-in GDL test confirms it in two built GDLs.
- Completed [task-12-r1](../phases/01-1025m-editor.prompts.md#task-12-r1): the exporter is found without setup (environment, preferences, sibling checkout) or chosen in the app, and the app builds GDL on a worker thread with a banner.
- Completed [task-13-r1](../phases/01-1025m-editor.prompts.md#task-13-r1): Git dependencies and embedded fonts are vendored, and the Windows CI checks use the root workspace and preserve the bundled font-index bytes.

## Verification evidence

- Task-09/10: the full `figcraft-engine`, `figcraft-ui-egui`, `figcraft-mcp` and `gui-studio-extron` suites, Clippy and rustfmt passed; the demo EXUI stayed `2B474E44…`.
- Task-11: the full `figcraft-engine` (2290 + 51) and `gui-studio-extron` (19) suites, Clippy and rustfmt passed. The ignored GDL test passed with `GUI_STUDIO_EXTRON_CLI` and `GUI_STUDIO_EXTRON_DONORS` set to the 0.8.8 stage kit (`../figma-extron-exporter/output/releases/0.8.8/stage-4811fda9ba304219a951713b18c28f6f`); its `smoke-kit` lacks prepared donors.
- Task-13: 3,895 offline tests passed across `figcraft-engine`, `figcraft-ui-egui`, and `figcraft-text`. The offline wasm check for `figcraft-web` and root formatting passed.

## Current working state

RUNNABLE / GREEN for the vendored build and the checks above. Dot-source `scripts/env.ps1` to use the Windows toolchain installed in `.tools/` by `scripts/bootstrap.ps1`.

## Blockers / uncertainty

- The new build was not launched: the Extron section, the badges and the alignment menu have only been exercised in the UI test harness. The running app was an older build holding an unsaved "Huddle Room" document.
- The GCP half of the identity criterion needs GCP, whose Import layout is blocked (study §13.5). No GDL from pages, kits or masters has been opened in GUI Designer.

## Next action

Recheck the saved Pencil import task against the Phase 01 plan and the current checkout.

## Active task prompt

`.progressive/phases/01-1025m-editor.prompts.md#task-04-r1`
