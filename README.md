# GUI Studio

GUI Studio is a Rust desktop editor for authoring Extron TouchLink Pro interfaces. Its first project type is a 1280 × 800 TLP Pro 1025M screen.

## Run on Windows

Open `target/debug/gui-studio.exe` after building, or use `cargo run --locked` from a Windows Rust development environment. The app opens with an editable meeting room demo. Use **File → New project** for an empty screen.

- Add buttons, sliders, labels, and shapes from the toolbar. Select layers on the canvas or from the layer list. Drag to move, or drag a selected corner to resize. Hold Shift during drag to snap the displacement to an 8 px step.
- Edit dimensions and appearance in the inspector. Select Normal, Active, or Pressed to edit a button state. The inspector shows the Extron Control ID for addressable controls.
- Scroll over the canvas to zoom. Drag with the middle mouse button to pan. **Fit** restores the initial view. In **Preview**, click buttons to toggle their sample feedback and move the slider to check the layout.
- Save or reopen an editable `.guistudio` project. **Export EXUI** writes a standalone EXUI/5 package.

## Build a GDL

Use **Export → Build GDL** for a new GDL file. This command uses the [Figma Extron Exporter](https://github.com/Elguajo/figma-extron-exporter), its donor registry, and the corresponding Extron GUI Designer installation. The editor discovers the working 0.8.8 smoke kit in a sibling exporter checkout. To use a different installation, select the CLI DLL and registry from the Export menu or set `GUI_STUDIO_EXTRON_CLI` and `GUI_STUDIO_DONOR_REGISTRY` to their local paths before launch.

Open the exported layout in Extron GUI Designer for **Verify** and **Save and Build**. Configure equipment actions and feedback in Global Configurator Professional.

## Development

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked --no-default-features -- --demo-exui aurora.exui
```

The specs and verification history live in [Progressive Context Kit](.progressive/project/PROJECT_BRIEF.md).
