//! Native TLP Pro 1025M authoring and EXUI/5 for GUI Studio.

mod adapter;
#[cfg(not(target_arch = "wasm32"))]
mod converter;
mod demo;
mod metadata;
mod model;
mod writer;

pub use adapter::project;
#[cfg(not(target_arch = "wasm32"))]
pub use converter::build;
pub use demo::document as demo;
pub use model::{Diagnostic, Project};
pub use writer::package;

#[cfg(test)]
mod tests;
