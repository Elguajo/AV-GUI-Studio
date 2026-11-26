# UI kits

A UI kit gives a touch panel project its look: the colours of native buttons (Normal, Active and Pressed), sliders, labels and shapes, the page background, and ready-made **blocks** of controls such as a source selection card or a volume control. A kit holds only what a native Extron control keeps in a GDL, so everything it makes exports without rasterising.

## Using kits

- **Built in:** Aurora (deep teal, the demo's look and the default), Daylight (light) and Graphite (dark grey with amber).
- **New project:** File → Extron → New touch panel project… asks for the touch panel and the kit.
- **UI kits window:** File → Extron → UI kits… shows every kit. **Controls** inserts a button, slider, label or shape in the chosen kit's look. **Blocks** inserts a block on the page of the selection, or the page in the middle of the view. **Use for this project** makes the kit the project's kit: the controls inserted from then on take its look, and placed controls keep theirs.
- **Masters:** buttons and sliders are instances of one master per kit, made beside the pages the first time a control of that kit is inserted (**Aurora button**, **Aurora slider**). Editing a master restyles every control made from it, whichever way it was inserted; after the master is edited, the kit's colours no longer describe those controls. A kit switch gives new controls the new kit's masters.
- **Install kit…** copies a kit file into the app's `UI Kits` folder, next to `Plugins` in the user's settings folder. A kit with the id of an installed kit replaces it. **Remove kit** deletes an installed kit; built-in kits stay.
- A project keeps a copy of its kit on its pages, so it opens with the same look on a computer where the kit is not installed.

The same actions are commands for the CLI and AI agents: `extron.kits`, `extron.installKit {path}`, `extron.removeKit {id}`, `extron.useKit {id}`, `extron.insertBlock {block, kit?, panel?, x?, y?}`, `extron.newPanel {kit?}` and `extron.insert {kit?}`. The CLI has no kits folder, so a kit it installs lasts for that run.

## Kit file

A kit is a UTF-8 JSON file of up to 256 KiB with the `.guikit` or `.json` extension. Unknown fields are refused. The built-in kits in [`crates/extron/kits/`](../crates/extron/kits/) are complete examples.

| Field | Content |
|---|---|
| `schema` | `"gui-studio-kit/1"` |
| `id` | 1–64 lowercase letters, digits and hyphens, starting with a letter. It can't be a built-in kit's id |
| `name` | 1–256 characters |
| `description`, `author` | Optional text |
| `background` | Page background, `#RRGGBB` |
| `button` | `normal`, `active` and `pressed`, each `{fill, text}` colours, and `size`, the caption size in points (4–200). The three states need different colours |
| `slider` | `track`, `fill` and `thumb` colours |
| `label` | `color`, `muted` (secondary text) and `size` in points |
| `shape` | `fill` and `radius` (0–255) |
| `blocks` | Up to 32 blocks, optional |

A block is `{name, description?, size: [width, height], items}`, with 1–64 items. Block names are unique within the kit. Each item is a control:

| Field | Content |
|---|---|
| `kind` | `button`, `slider`, `label` or `shape` |
| `name` | The layer name, also the caption when there is no `text` |
| `text` | Caption of a button or label, optional |
| `rect` | `[x, y, width, height]` from the block's top left, inside the block |
| `size` | A label's size in points instead of the kit's, optional |
| `muted` | `true` for a label in the kit's `muted` colour, optional |

A block is inserted only where it fits the page: a 1280 px wide block fits a TLP Pro 1025M but not a 320 × 480 TLP Pro 300M. Each control gets a new key and Object ID, and a button or slider a Control ID. A page holds up to 64 controls.
