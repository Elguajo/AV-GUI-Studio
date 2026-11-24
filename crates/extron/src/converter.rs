//! Run the configured external exporter and publish only a verified GDL.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

use crate::{model::Project, package};

const CLI_ENV: &str = "GUI_STUDIO_EXTRON_CLI";
const REGISTRY_ENV: &str = "GUI_STUDIO_EXTRON_DONORS";

fn config_path(variable: &str) -> Result<PathBuf, String> {
    let path = std::env::var_os(variable).map(PathBuf::from).ok_or_else(|| format!("Set {variable} to an absolute file path."))?;
    if !path.is_absolute() || !path.is_file() {
        return Err(format!("{variable} must point to an existing file at an absolute path."));
    }
    Ok(path)
}

pub fn build(project: &Project, output: &Path) -> Result<Value, String> {
    let cli = config_path(CLI_ENV)?;
    let registry = config_path(REGISTRY_ENV)?;
    let parent = output.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    if output.try_exists().map_err(|error| error.to_string())? {
        return Err(format!("{} already exists. Choose a new GDL filename.", output.display()));
    }

    let directory = tempfile::tempdir_in(parent).map_err(|error| error.to_string())?;
    let exui = directory.path().join("screen.exui");
    let gdl = directory.path().join("screen.gdl");
    fs::write(&exui, package(project)?).map_err(|error| error.to_string())?;
    let mut command = Command::new("dotnet");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let result = command
        .arg(cli)
        .arg("convert-package")
        .arg(&exui)
        .arg(&gdl)
        .arg("--registry")
        .arg(registry)
        .output()
        .map_err(|error| format!("Could not launch the Extron exporter: {error}"))?;
    let stdout = String::from_utf8_lossy(&result.stdout);
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("Extron exporter returned {}. {} {}", result.status, stderr.trim(), stdout.trim()));
    }
    let report: Value = serde_json::from_str(&stdout).map_err(|error| format!("Could not read the Extron conversion report: {error}"))?;
    let checks = report["checks"]
        .as_str()
        .filter(|checks| {
            checks
                .strip_suffix(" passed")
                .and_then(|value| value.split_once('/'))
                .is_some_and(|(passed, total)| passed.parse::<u32>().is_ok_and(|passed| passed > 0 && total.parse::<u32>() == Ok(passed)))
        })
        .ok_or("The Extron report did not confirm that all checks passed.")?;
    let reader = fs::File::open(&gdl).map_err(|error| error.to_string())?;
    let mut staging = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    std::io::copy(&mut std::io::BufReader::new(reader), staging.as_file_mut()).map_err(|error| error.to_string())?;
    staging.as_file().sync_all().map_err(|error| error.to_string())?;
    staging.persist_noclobber(output).map_err(|error| format!("{}: {error}", output.display()))?;
    Ok(serde_json::json!({"path": output, "checks": checks}))
}
