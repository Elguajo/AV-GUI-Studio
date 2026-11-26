# Vendored sources

Every source GUI Studio builds from, apart from crates.io packages, is a copy in this folder. The
build reads nothing from other Git repositories: the root `Cargo.toml` names these copies as `path`
dependencies and patches, and `.cargo/config.toml` points `CRAFT_FONTS_DIR` at `craft-fonts`.
crates.io packages stay on crates.io, at the versions `Cargo.lock` records.

Each copy holds only the crates this build uses, taken unchanged from the commit below, with each
project's licence files. The workspace is set to skip this folder (`exclude` in the root
`Cargo.toml`), so `cargo fmt`, Clippy and the tests leave it alone.

| Folder | Source | Commit | What is here |
| --- | --- | --- | --- |
| `craft-libs` | [storytold/craft-libs](https://github.com/storytold/craft-libs) | `a49329e71e32ad93794ab595c41f33209a7d2220` | 30 `crates/*` and `vendor/krilla` (krilla 0.8.2 with craft-libs' changes) |
| `craft-libs/crates/{dds,image-codecs,jpeg,metadata,tiff}` | storytold/craft-libs | `ba25ccd233934d1b2f74e3c0f235168bf5f13ef3` | see below |
| `craft-libs-ea9407b` | storytold/craft-libs | `ea9407b3500488bd8af71eb0f1b950306315bf7e` | `crates/web-api`, `crates/web-host` (the web build only) |
| `egui` | [storytold/egui](https://github.com/storytold/egui) | `6d518fc799681571963842f95b55a6b68431b15a` | ecolor, emath, epaint, epaint_default_fonts, egui, egui-winit, egui-wgpu, egui_glow, egui_extras, eframe |
| `winit` | [storytold/winit](https://github.com/storytold/winit) | `a517b3846ee8457d84ee4975110bfda6797c44dc` | winit 0.30.13 and its `dpi` crate |
| `vectorcraft` | [storytold/vectorcraft](https://github.com/storytold/vectorcraft) | `6b3a6b8788ebcb420d69449199cbdf323888d27e` | `crates/geom`, `crates/pathops` (used by craft-libs) |
| `craft-fonts` | [storytold/craft-fonts](https://github.com/storytold/craft-fonts) | `37827adc41fd165f9a9234ac40f94dbad353aa69` | `fonts/` with its manifests, `ATTRIBUTION.md`, licences |

Licences: the code is MIT OR Apache-2.0 (`LICENSE-MIT` and `LICENSE-APACHE`, and `NOTICE` where
the project has one, in each folder; winit is Apache-2.0, `winit/LICENSE`). The fonts are under the SIL Open Font License 1.1 and
the other licences listed per font in `craft-fonts/fonts/manifest.txt` and
`craft-fonts/fonts/latin-manifest.txt`, each with its licence file beside the font.

## Changes from the sources

- Workspace manifests (`craft-libs/Cargo.toml`, `craft-libs-ea9407b/Cargo.toml`, `egui/Cargo.toml`,
  `vectorcraft/Cargo.toml`, `winit/Cargo.toml`): `[patch]` tables removed (only the root workspace's
  patches ever applied), workspace dependencies on other Git repositories removed, `members` cut to
  the crates that are here. In the craft-libs manifests, `vectorcraft-geom` and
  `vectorcraft-pathops` point at `../vectorcraft`.
- `craft-libs/crates/{dds,image-codecs,jpeg,metadata,tiff}` are from `ba25ccd`, a later craft-libs
  commit, for every crate that uses them. GUI Studio took `craft-image-codecs` from `ba25ccd` and the
  rest of craft-libs from `a49329e`; two path copies of one crate version cannot share a
  `Cargo.lock`. Between the two commits these crates differ only in `image-codecs`, which gained
  PNG physical-resolution reading and writing; nothing public was removed.
- `egui/crates/egui-winit`: no repaint when the window moves; see its `VENDORED.md`.
- `.gitattributes` preserves the bytes of `craft-libs/crates/font-store/index/` on checkout so the
  bundled font indexes match their pinned SHA-256 hashes on Windows.
- Repository tooling (CI, `.cargo`, `rust-toolchain`) and the crates this build does not use are
  left out.

## Updating a copy

Take the new commit's crate folders whole, re-apply the changes above, update the table, and run
`cargo metadata` once so `Cargo.lock` follows. Then run the full checks (`cargo xtask ci`).
