//! Native Extron touch panel authoring, UI kits and EXUI/5 for GUI Studio.

mod adapter;
mod author;
mod control;
#[cfg(not(target_arch = "wasm32"))]
mod converter;
mod demo;
mod kit;
mod metadata;
mod model;
mod writer;

pub use adapter::{is_screen, project};
pub use author::{
    Assigned, ControlKind, Insert, KIT_KEY, MASTER_KEY, NEXT_ID_KEY, NewPage, SyncInstance, add_page, assign, insert, insert_block, panel,
    panel_frame, panel_of, panels, project_kit, project_profile, set_kit,
};
pub use control::{ControlEdit, ControlInfo, MAX_CONTROL_ID, control_ids, control_info, edit_control, kind_name};
#[cfg(not(target_arch = "wasm32"))]
pub use converter::{
    CLI_ENV, EXPORTER_VERSION, Exporter, ExporterSource, REGISTRY_ENV, REGISTRY_ENV_OLD, build, check_exporter, find_exporter, missing_donor,
};
pub use demo::document as demo;
pub use kit::{Block, BlockItem, DEFAULT_KIT, KIT_SCHEMA, Kit, MAX_KIT_BYTES};
pub use metadata::Kind as ElementKind;
pub use model::{Color, Control, DEFAULT_PROFILE, Diagnostic, Element, PROFILES, Page, Profile, Project, StateRole, profile};
pub use writer::package;

#[cfg(test)]
mod author_tests;
#[cfg(test)]
mod tests;
