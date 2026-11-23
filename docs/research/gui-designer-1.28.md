# Extron GUI Designer 1.28 — functional study

Date: 2026-11-23. Source: GUI Designer 1.28.0-b.7 installed locally, its bundled help (`Help/Content/HTML_Files`), and a hands-on pass over a new, unsaved TLP Pro 1025M project from the Mach theme template. The study is black-box: no binaries were decompiled or inspected. Facts are marked **[help]** or **[observed]**. Inferences are marked **[inference]**.

## 1. Workspace

- Docking WinForms layout **[observed]**:
  - left: Controls palette and Workspace tree;
  - center: Design window with page tabs;
  - right: Properties grid;
  - bottom: Control States strip;
  - auto-hide side tabs: Resource Library and Where Used.
- Panels float, dock and tab. Window Layout Presets and Workspace Presets save and restore layouts and open-project sets **[help]**.
- The Easy Assist toolbar is one long icon strip for file, edit, view toggles, Verify, Character Map, Font dialog and roughly 30 align, size and spacing icons **[observed]**.
- Status bar **[observed]**:
  - current page;
  - Panel RAM Usage gauge, which estimates whether the project fits the panel **[help]**;
  - signed-in account;
  - zoom slider with a percentage.
- Several projects can be open at once. Pages and objects can be copied between projects. Pasting resolves duplicate resources with Add, Replace, Skip or Cancel **[help]**.

## 2. Project structure

- New Project wizard **[observed]**:
  1. Pick a panel type: TLP Pro, TLC Pro, TLI Pro, Extron Control for iOS, Android and Web, and CCI Pro.
  2. Pick Blank or a Theme. A theme takes a Theme name (for example Mach or Jet) and an Application (Single Display, Dual Display, Multi Window, Divisible Room, Video Conference).
  3. Pick a Project Font.

  Some platforms offer a choice of resolution.
- Workspace tree **[observed]**:
  - Project root.
  - Offline Page, shown when the controller connection is lost. The project property Enable Offline Page turns it on or off.
  - Pages, shown in orange. A page lists its popup page references as "Controls" children.
  - Popup pages, shown in blue with a window icon.
  - Modal popup pages, shown in blue with a plain icon **[inference from icons and the help's three Add commands]**.
- Every page, popup and modal popup has a unique name **[help]**.
- Popup groups: a popup sets a Group name, and popups in the same group are mutually exclusive. A popup page reference points to either one popup or a group **[help]**.
- Project Properties **[help]**:
  - Design tab: default start page, project font, platform type, which can be changed to retarget the project, and dimensions or scaling.
  - Information tab: name, job number, PO number, version Major.Minor.Build with Build incremented on every save, and comments.
  - Contacts tab.

## 3. Controls (native types)

| Palette item | Shortcut | Notes |
|---|---|---|
| Button | Ctrl+F3 | States, ID, text and image per state, border resource, Transparent Pixels = Pressable/… |
| Level | Ctrl+F4 | Display Type: Gauge, Slider Indicator, Multistate, Volume LED, None. Orientation. Tick marks. Bar image or color |
| Slider | Ctrl+F4 (help) | Orientation; fill, track, thumb (ellipse/rectangle/image/none) colors, images and sizes; ID |
| Label | Ctrl+F8 | Text, font, shape None/Rectangle/Ellipse, 2D/3D style, corner radius; ID for dynamic text |
| Image | Ctrl+F7 | Graphic resource, layout and alignment, key color, optional shape |
| Shape | Ctrl+F11 | Border resource, border color, fill, transparency |
| Line | Ctrl+F10 | Start and end points, caps, thickness |
| Date/Time | Ctrl+F6 | Pattern, font, shape |
| Popup Page Reference | Ctrl+F5 | Popup or group, geometry. Can be created by dragging a popup from the tree |
| Video, Dynamic Image | Ctrl+F9 | Only on specific models. Not part of the 1025M EXUI/5 contract |

Insertion: click a palette item, then drag it onto the page **[help]**.

## 4. Properties model

The Properties panel is a categorized property grid with an object picker at the top and a description pane below. It offers alphabetical, expand-all and collapse-all views **[observed]**. Categories **[help, observed]**:

- **Appearance**:
  - Text and Use Flattened Text.
  - Font, style, size, text color and alignment.
  - Image, Image Layout (Fill/Stretch/…), Image Alignment and Image Key Color, the transparent color.
  - Border Type, a resource from the library such as 2D Rounded Rect. Border Color, which may carry alpha. Fill Color and Transparency in %.
- **Behavior**: Transparent Pixels, which sets how invisible pixels react to touch.
- **Design**: Name, and ID for buttons, levels, sliders and labels.
- **Layout**: Left, Top, Width, Height.
- **Misc**: Description, also shown in Where Used.
- **States**: an indexed list such as `[0] Off` and `[1] On`. Each state expands into its own appearance sub-properties.

Defaults: editing properties with a palette item selected changes that type's defaults for new objects **[help]**.

## 5. States

- The Control States strip shows one thumbnail per state, with its index and ID badge **[observed]**. Its commands are Add with a count, Insert before, Move Left or Right with a count, Delete, Select All and its own undo **[help]**.
- Buttons default to two states. In the help, `[0]` is Off (not pressed) and `[1]` is On (pressed). In the Mach theme, Mute's state 1 is an orange feedback look **[observed]**. A multistate Level has N states **[help]**.

## 6. ID semantics (the core of Extron identity)

- Buttons, levels, sliders and labels need an ID for GCP **[help]**.
- **Sharing an ID means sharing function.** Objects with the same ID get the same GCP actions. Modifications such as added states propagate across objects that share an ID **[help]**.
- IDs can be shared only within the same control type. Buttons, and likewise levels, must also have the same number of states. Otherwise GUI Designer refuses with a named reason and highlights the ID field **[help]**.
- The ID field offers None, First Available, and the IDs already used by that control type **[help]**.
- Copy and Paste give the copy the first free ID if the original still exists. Cut and Paste keep the ID. Paste (keep IDs the same), Ctrl+Alt+V, keeps the ID deliberately **[help]**.
- Tools **[help]**:
  - Auto-Assign Object IDs for a selection: specified start or First Available, with optional contiguous IDs and reuse of current IDs.
  - Offset Object IDs (F6), which starts new IDs from a base.
  - A preference to auto-assign an ID on insert.
  - Show Control Info, which draws yellow ID badges on every object, or the reference name for popup references **[observed]**.
  - The Where Used panel.
- Theme template IDs fall in function blocks such as 8050–8063 for sources, 8112–8116 for display and settings, and 5150–5164 for document camera **[observed]**. The appendix holds standard ID tables per device class (DVD, VCR, camera, VTC and others) used by GCP Auto Fill **[help]**.

## 7. Resources

- Resource Library tabs **[observed]**:
  - Graphics: 80 items in the Mach project, mostly 440×440 icon PNGs and 1376×440 headers.
  - Borders: editable resources made in the Border Editor. Editing a border changes every user.
  - Fonts.
- Operations **[help]**: import a file or folder, export a selection or a resource set (`.grs`), rename, delete, select unused, and strip extensions on import. Save and Build can warn about, remove or keep unused resources.
- Font Replacement (F5) swaps a font project-wide. Character Map inserts icon-font glyphs into text **[help]**.
- Custom palette colors are managed in a Palette Manager **[help]**.

## 8. Layout tools

- Align **[help]**:
  - to selection: lefts, horizontal centers, rights, tops, vertical centers, bottoms;
  - to page: top, vertical center, bottom, left, horizontal center, right;
  - to grid.
- Sizing to the first selected object: same width, same height or same size. Also Size to Image and Size to Text **[help]**.
- Horizontal and vertical spacing: make same (average), increase, decrease, remove **[help]**.
- Arrange: front, forward, backward, back, with Alt++, +, - and Alt+- **[help]**.
- Grid **[help]**: show, size and snap. Snap lines are a Preference. Arrow keys nudge by "Move Objects by", 1–250 px.
- Marquee direction matters **[help]**: dragging right to left selects touched objects, and dragging left to right selects only enclosed objects.
- Lock locks an object's or a page's properties **[help]**.
- Show Object Layer Indicator marks objects that cover other objects **[help]**.
- Hide Popup Page References lets you edit the page under them **[help]**.

## 9. View and quality aids

- Zoom changes in 10% steps with Ctrl+plus and Ctrl+minus. Ctrl+0 fits, Ctrl+Alt+0 goes to 100%, and there is a Last Zoom command. Holding Ctrl+Shift turns the cursor into a magnifier for wheel zoom. The status bar has a zoom slider **[help]**.
- Monochrome Mode removes color so that low contrast shows up, as an accessibility check **[help]**.
- Snapshots export the current page, all pages, or all page and popup permutations as images **[help]**.

## 10. Verify and build

- Verify, or Save and Build with Ctrl+Shift+B, opens the Build Manager. It lists errors and warnings, and **Show Me** or a double-click jumps to the cause. The list is not refreshed until the next build **[help]**.
- Where Used **[help, observed]**:
  - tabs: Controls (ID, type, name, page, description), Popup Pages, Pages, Popup Groups;
  - search, type filter, and Show Only Duplicate IDs;
  - export to PDF;
  - inline renaming;
  - double-clicking an item selects it on the canvas.

## 11. Observed interaction quality (opportunities for GUI Studio)

- **Drag.** The object moves live and shows blue snap lines on edges aligned with siblings. Use Snap Lines is on by default. No distance readout appears, and spacing is not shown **[observed, second pass]**. In the first pass the canvas did not repaint during the drag.
- **Property grid.** All editing happens in a generic WinForms grid. Colors are RGB triplets. States are edited by expanding sub-properties, and no canvas toggle previews a state **[observed]**.
- **Inserting objects.** Insertion is a two-step palette-then-drag mode. Zoom moves in coarse 10% steps **[help]**.
- **ID badges.** Badges overlap the object's text **[observed]**.
- **Validation.** Errors appear only on Verify or Build, in a separate dialog, and the list goes stale until the next build **[help]**.
- **Visual language.** Dense 2000s toolbar iconography with 30+ alignment icons in one strip. The panels have no visual hierarchy **[observed]**.

## 12. Parity map for GUI Studio

| GUI Designer capability | GUI Studio approach | Phase |
|---|---|---|
| Canvas, selection, nudge, arrange, lock, zoom | Figma interaction core (ADR-001). Marquee follows Figma (touch selects). Arrow nudge 1/10 px | 01 |
| Align, distribute, match size | Figma-style align and distribute in the inspector header, plus "match size to first selected" | 01 |
| Show Control Info | Control ID badges outside the text area, toggled with one key | 01 |
| Verify / Build Manager / Show Me | Live Problems panel that updates on every edit, click to select | 01 |
| Pages, popups, modal popups, popup groups, popup references, Offline Page, start page | Pages panel with the same constructs | 02 |
| Hide popup references, layer indicator | Visibility toggles in Layers, plus a "show references" toggle | 02 |
| ID sharing with type and state-count compatibility; Paste keep IDs; Auto-assign; Offset; First Available | ID manager with explicit "Link to existing ID" and the same compatibility rules, reported in Problems | 03 |
| Where Used (search, duplicates, export) | Searchable ID table panel; export later | 03 |
| Resource Library (graphics, borders, fonts, palette, unused cleanup, resource sets) | Assets panel: images, border styles, fonts, color variables | 03 |
| Copy and paste properties with a picker | Ctrl+Alt+C / Ctrl+Alt+V with a property picker | 03 |
| Font Replacement, Character Map icon fonts | Text styles plus a font swap; icon-font glyph picker | 03 |
| Snapshots and permutations, Monochrome check, RAM usage estimate | Export PNG, grayscale preview toggle, size estimate in the status bar | 04 |
| Project metadata (job, PO, version, contacts) | Project settings | 04 |
| Themes and templates | Starter projects built from components | Later |
| Video, Dynamic Image, non-1025M platforms | Outside the current EXUI/5 contract | Later |

## 13. Experiments, second pass (2026-11-23)

These experiments ran on scratch copies only. The GDL inputs were built from `Project::demo()` by exporter CLI 0.8.8. The independent verifier passed 182/182 on `studio.gdl`, with SHA-256 `857C5161…056E`.

### 13.1 Round-trip of a GUI Studio GDL

- GUI Designer opened the file and marked it modified at once. Verify reported **0 errors, 0 warnings**. **Save and Build succeeded.** The container changed from a single entry to a multi-entry file that adds `<name>.tgz4`, about 800 KB of built panel output **[observed]**. Opening it in GCP requires this built form, as the exporter's GCP protocol from 2026-10-23 records.
- `compare-graph` on the original and the re-saved file found no change to IDs, names, states, geometry or colors. GUI Designer changed only these fields **[observed]**:
  - Project build metadata: `versionInfo` 0.0.1 → 0.0.2, revision date, build path, type and status, and the preview image.
  - `TLPImageID` on pages, controls and states: −1 became assigned render IDs. Identical renders share one ID, so USB-C and Wireless reuse 9/10/11.
  - **`zOrder` was renumbered to the list index on all 14 controls.** The exporter writes zOrder values such as 0, 8 and 10 that do not match the list order. GUI Designer treats list order as the z-order.
- Save and Build with **Remove unused resources** took the project payload from 2.49 MB to 2.11 MB. It dropped 31 of 39 `PBImageResource`, 4 of 34 borders and 1 of 3 fonts. This matches the exporter's documented design. The writer carries over the donor's resources and palette, and Remove cleans out the ones the layout does not use (`figma-extron-exporter/docs/extron-format.md`). The zOrder renumbering is not covered in the exporter's documentation or source.
- The GUI Designer **ID** field is the **Control ID**. Changing Wireless's ID to 8120 left its Object ID at 8110 **[observed via `list-controls`]**. Object ID never appears in the GUI Designer UI.

### 13.2 Validation catalog

| Action | Where GUI Designer reports it | Message |
|---|---|---|
| Give a button the ID of a slider | Dialog when the ID is committed | "The ID you entered is currently being used by a control of a different type. Only controls of the same type can share IDs." |
| Give a 2-state button the ID of a 3-state button | Dialog when the ID is committed | "…currently being used by an incompatible button control. The number of Control States has to be the same." |
| Rename a popup to an existing page name | Dialog when the name is committed | "Duplicate page or popup page names are not allowed within the same project." |
| Popup with no reference | Verify and Build **warning** | "This popup page has no references so it can never be shown/hidden." |
| Set Left so the object leaves the page | Silently clamped (Left 1250 → 1079 for width 201) | — |
| Duplicate **object** name, empty (Unassigned) popup reference, shared ID between compatible buttons | Accepted. Verify and Build stay clean | — |

- Sharing an ID between compatible buttons synchronizes the **state count**: adding a state to one added a state to the other. **Appearance is not synchronized**: font size 30 on one left the other at 21 **[observed]**.
- State names follow the object name, as in `[0] HDMI - Normal`. Renaming the object renames its states **[observed]**.
- A new button gets the first free ID (1, then 2), two states, Arial 12 and a size of 139×70 when drawn or 100×70 when clicked. The insert tool stays active after placement **[observed]**.
- Double-clicking a button opens inline text editing. Undo history lists named commands such as "Change Home.USB-C.Text" **[observed]**.
- A new modal popup is full-panel with black fill at 35% transparency. It comes with a title label and a Close button. In the Workspace tree a plain square icon marks a modal popup and a window icon marks a regular popup **[observed]**.

### 13.3 Object model facts from the shipped `GUI Designer.xml`

The installed XML documentation (9,900 members) is read as documentation. No code was inspected.

- **Platforms:** 55 classes, covering TLP Pro 300M–1725TG, TLC Pro, TLI Pro, ZRTP and virtual iOS, Android, Web and ECP. `PBTouchPanelPlatformPro` exposes `SupportedResolutions`.
- **State (`PBState`)** has two image layers: `BackgroundImage*` and `ButtonImage*`. Each layer has layout, left and top offsets, alignment and key color. A state also has text, `FText` for flattened text, font family, style and size, text color and alignment, `Border`, `BorderColor`, `BorderFillColor`, `Transparency` and **`Blinking`**.
- **Button:** `TLPPressFeedbackStateID`, `TLPDefaultStateID`, `HidePressFeedback`, `HideVisualFeedback`, `HideTextFeedback`.
- **Image layout:** Fill, Stretch, Tile, Native, Offset. **Key color:** `Color`, `Tolerance`, `Enabled`, `StartColor`, `EndColor`.
- **Border Editor** **[observed]**: Shape None/Rectangle/Ellipse, style 2D/3D, corner radius and thickness in px. 3D adds light brightness in %, light angle, frame depth and surface height. The library ships 35 borders, from 2D/3D Capsule, Ellipse and Rectangle to Rounded Rectangle, Gradient and their Selected variants.
- **Design View defaults** **[observed]**: Use Snap Lines on, Snap to Grid off, 8×8 grid shown, Move Objects by **10 px**.

### 13.4 Reference render

[`assets/gd-1.28-aurora-home-1025m.png`](assets/gd-1.28-aurora-home-1025m.png) is GUI Designer's 1280×800 snapshot of the demo Home page, with SHA-256 `E8EB0665…2BA3`. It is the pixel reference for GUI Studio's canvas. Show Control Info must be off before taking a snapshot, because snapshots include the ID badges.

### 13.5 GCP binding experiment — blocked

- A test project was set up in GCP Plus 3.34.1.4 with an IPL Pro CR88 at 192.0.2.10 and a TLP Pro 1025M at 192.0.2.11. The built v1 path was given in the add-panel dialog, but the panel showed only the Bezel page.
- The Import layout icon in Layout view opened no dialog. Mouse clicks and an explicit press and release made no difference.
- A built v2 is ready at the scratch path for the comparison. Its changes, checked with `list-controls`: USB-C renamed to "USB-C Input" with the same ID, HDMI with 4 states instead of 3, and Wireless Control ID 8110 → 8120 with Object ID 8110 kept.
- Still to verify: Layout Reconciliation results for a rename, a state-count change and a Control ID change, and a slider binding.

### 13.6 Remaining open questions

- Transparent Pixels values other than Pressable. The drop-down could not be opened in this pass.
- GCP behavior from 13.5.
