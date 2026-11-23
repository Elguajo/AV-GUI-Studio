#![cfg_attr(all(windows, feature = "desktop"), windows_subsystem = "windows")]

use std::{env, path::Path, process::ExitCode};

use gui_studio::{exui, project::Project};

#[cfg(feature = "desktop")]
mod studio;

fn main() -> ExitCode {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let result = match args.as_slice() {
        [command, path] if command == "--demo-exui" => {
            exui::save(&Project::demo(), Path::new(path))
        }
        [command, project, path] if command == "--export-exui" => Project::open(Path::new(project))
            .and_then(|project| exui::save(&project, Path::new(path))),
        [] => launch(),
        _ => Err("Usage: gui-studio [--demo-exui OUTPUT | --export-exui PROJECT OUTPUT]".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "desktop")]
fn launch() -> Result<(), String> {
    studio::run().map_err(|error| error.to_string())
}

#[cfg(not(feature = "desktop"))]
fn launch() -> Result<(), String> {
    Err("Build with the desktop feature to open the editor.".into())
}
