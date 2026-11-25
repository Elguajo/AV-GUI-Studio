# GUI Studio

![GUI Studio app icon](assets/app-icon/hicolor/128x128/apps/ai.storyteller.figcraft.png)

GUI Studio is a Rust desktop editor for Extron touch panel layouts. The local Windows build supports authoring and exporting one TLP Pro 1025M screen.

## Build and run

Use the Windows Rust GNU toolchain and make the MinGW C compiler, headers, CRT libraries and resource tools available. From this checkout:

```powershell
cargo build --locked --release -p figcraft -p figcraft-cli
.\target\release\gui-studio.exe
```

Choose **File → Extron → New 1025M demo** to open the editable Aurora Meeting Room example. Its **Home** frame contains labels, shapes, a slider and instances of the source and Mute button sets. Edit button state fills and captions on the variants beside the screen. Use **Validate screen**, **Export EXUI** or **Build GDL** in the same menu.

**File → Extron → New touch panel project…**, or the card on the home screen, first asks for the touch panel, as GUI Designer's New Project wizard does: TLP Pro 1025M (1280 × 800) or TLP Pro 300M portrait (320 × 480), the two panels the exporter builds GDL for, and then for a UI kit. The project opens with an empty **Home** page of that resolution.

UI kits give native controls their look and offer ready-made blocks, such as a source selection card or a volume control. Aurora, Daylight and Graphite are built in; **File → Extron → UI kits…** inserts controls and blocks, switches the project's kit and installs kit files. See [UI kits](docs/ui-kits.md) for the kit file format.

Each page of a project is an artboard of the panel's size on the canvas. The **Artboard** tool (Shift+O, beside the frame tools) adds one: click to put its top-left corner there, or drag to place it. Enter with the tool, or **File → Extron → Add page**, adds one to the right of everything on the canvas. The page with the lowest Object ID, Home, is the start page. A project of more than one page exports as an EXUI/5 project whose other pages are shown by the control program.

Controls are added with the `extron.insert` and `extron.assign` commands, from the CLI or an AI agent (see below); the editor has no toolbar for them yet. `extron.insert` takes the page of the selection, or the page in the middle of the view. Each page and control gets an Object ID, and a button or slider the same Control ID, from one counter stored on every page, so the ID of a deleted control or page is never handed out again.

The **Problems** section under the layers lists every export problem of an Extron screen and updates as you edit. Select a problem to select its layer, reveal it in the layers and bring it into view. Export stays blocked until the list is empty.

EXUI export works on its own. Configure the separately distributed Extron exporter 0.8.8 and donor registry before starting the editor or CLI if you plan to build GDL:

```powershell
$env:GUI_STUDIO_EXTRON_CLI = 'D:\DEV\figma-extron-exporter\output\releases\0.8.8\smoke-kit\cli\Extron.Cli.dll'
$env:GUI_STUDIO_EXTRON_DONORS = 'D:\DEV\figma-extron-exporter\samples\donors.json'
```

Use a new output filename for GDL. The app publishes the output after the converter reports all independent verification checks passed.

## Headless export

Run `gui-studio-cli commands --filter extron` for the registered command IDs and parameters. Save the following command array as `aurora.json`, then run `gui-studio-cli run aurora.json`:

```json
[
  {"command":"extron.demo"},
  {"command":"extron.validate"},
  {"command":"file.saveAs","params":{"path":"aurora.figcraft"}},
  {"command":"extron.exportExui","params":{"path":"aurora.exui"}},
  {"command":"extron.buildGdl","params":{"path":"aurora.gdl"}}
]
```

Pass `--in aurora.figcraft` to run against a saved design. `extron.validate` returns each problem with its `nodeId`, `layer` and `message`. A blocked export fails with one line per problem, such as `"Room title" (node 12): Element must fit inside the 1280 × 800 screen.` Root-confined automation can validate and export EXUI. Building GDL runs an external process and is available from the local desktop and an unconfined CLI run.

To author a screen without the editor, start from `extron.newPanel` and add controls:

```json
[
  {"command":"extron.newPanel","params":{"name":"Huddle Room"}},
  {"command":"extron.insert","params":{"kind":"label","text":"Huddle Room","x":60,"y":60}},
  {"command":"extron.insert","params":{"kind":"button","text":"Mute","x":60,"y":300}},
  {"command":"extron.insert","params":{"kind":"slider","x":300,"y":312}},
  {"command":"extron.exportExui","params":{"path":"huddle.exui"}}
]
```

`extron.assign {kind, ids?}` turns layers drawn directly in the panel into controls: a text layer into a label, a rectangle into a shape, a frame with a track, fill and thumb into a slider, and an instance of a three-variant set into a button whose variants become Normal, Active and Pressed in order.

## AI agents (MCP)

`gui-studio-cli mcp` is an MCP server for Claude Code, Codex and other MCP clients. This repository carries [`.mcp.json`](.mcp.json) for Claude Code and [`.codex/config.toml`](.codex/config.toml) for Codex, each with a headless server and one attached to the app started with `gui-studio --control 7979`. Build the CLI first; setup and the tools are in [docs/mcp.md](docs/mcp.md#configure-a-client).

## License

GUI Studio is an independent product built on a snapshot of [FigCraft](https://github.com/storytold/figcraft) `b828e6c7`, which is licensed MIT OR Apache-2.0. Selected FigCraft changes are ported by hand ([ADR-003](.progressive/decisions/ADR-003-independent-repository.md)). See [NOTICE](NOTICE), [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE) and [asset provenance](ATTRIBUTION.md).

## Project specs

Product brief, architecture, decisions and phase plans are in [Progressive Context Kit](.progressive/project/PROJECT_BRIEF.md). The original eframe prototype is kept at tag `eframe-prototype`.
