//! Native Extron touch panel authoring, UI kits and EXUI/5 for GUI Studio.

mod adapter;
mod author;
#[cfg(not(target_arch = "wasm32"))]
mod converter;
mod demo;
mod kit;
mod metadata;
mod model;
mod writer;

pub use adapter::{is_screen, project};
pub use author::{
    Assigned, ControlKind, Insert, KIT_KEY, NEXT_ID_KEY, NewPage, add_page, assign, insert, insert_block, panel, panel_frame, panel_of, panels,
    project_kit, project_profile, set_kit,
};
#[cfg(not(target_arch = "wasm32"))]
pub use converter::build;
pub use demo::document as demo;
pub use kit::{Block, BlockItem, DEFAULT_KIT, KIT_SCHEMA, Kit, MAX_KIT_BYTES};
pub use model::{DEFAULT_PROFILE, Diagnostic, PROFILES, Page, Profile, Project, profile};
pub use writer::package;

#[cfg(test)]
mod author_tests;
#[cfg(test)]
mod tests;
