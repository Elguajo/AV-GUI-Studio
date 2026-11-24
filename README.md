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

Pass `--in aurora.figcraft` to run against a saved design. Root-confined automation can validate and export EXUI. Building GDL runs an external process and is available from the local desktop and an unconfined CLI run.

## License

GUI Studio is a downstream fork of FigCraft. See [NOTICE](NOTICE), [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE) and [asset provenance](ATTRIBUTION.md).
