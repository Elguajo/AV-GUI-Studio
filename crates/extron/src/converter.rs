//! Find the separately distributed Extron exporter, run it, and publish only a verified GDL.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

use crate::{model::Project, package};

pub const CLI_ENV: &str = "AV_GUI_STUDIO_EXTRON_CLI";
pub const REGISTRY_ENV: &str = "AV_GUI_STUDIO_EXTRON_DONORS";
/// The names from before the rename to AV GUI Studio, still read after [`CLI_ENV`] and
/// [`REGISTRY_ENV`]; the registry's last one is the eframe prototype's.
pub const CLI_ENV_OLD: &str = "GUI_STUDIO_EXTRON_CLI";
pub const REGISTRY_ENV_OLD: &[&str] = &["GUI_STUDIO_EXTRON_DONORS", "GUI_STUDIO_DONOR_REGISTRY"];
/// The exporter release AV GUI Studio is tested with, looked for in a sibling checkout.
pub const EXPORTER_VERSION: &str = "0.8.8";

/// Where the exporter came from: the environment, the app's settings or a sibling checkout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExporterSource {
    Environment,
    Settings,
    Sibling,
}

impl ExporterSource {
    pub fn name(self) -> &'static str {
        match self {
            Self::Environment => "environment",
            Self::Settings => "settings",
            Self::Sibling => "sibling",
        }
    }
}

/// The exporter's CLI (`Extron.Cli.dll`, run with `dotnet`) and its donor registry (`donors.json`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Exporter {
    pub cli: PathBuf,
    pub registry: PathBuf,
    pub source: ExporterSource,
}

/// Check a CLI and registry pair: both are files at absolute paths, and the registry lists donors
/// with at least one donor file in place.
pub fn check_exporter(cli: &Path, registry: &Path, source: ExporterSource) -> Result<Exporter, String> {
    for (path, what) in [(cli, "The exporter CLI"), (registry, "The donor registry")] {
        if !path.is_absolute() || !path.is_file() {
            return Err(format!("{what} {} is not a file at an absolute path.", path.display()));
        }
    }
    let donors = donors(registry)?;
    if !donors.iter().any(|(_, file)| file.is_file()) {
        let id = donors.first().map_or("tlp-pro-1025m", |(id, _)| id.as_str());
        return Err(format!("No donor GDL of {} is prepared. Run the exporter's prepare-donor {id} first.", registry.display()));
    }
    Ok(Exporter { cli: cli.to_path_buf(), registry: registry.to_path_buf(), source })
}

/// The registry's donors: id and the GDL file it names, relative to the registry.
fn donors(registry: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let text = fs::read_to_string(registry).map_err(|e| format!("{}: {e}", registry.display()))?;
    let json: Value = serde_json::from_str(&text).map_err(|e| format!("{} is not a donor registry: {e}", registry.display()))?;
    let base = registry.parent().unwrap_or(Path::new("."));
    let list = json["donors"].as_array().ok_or_else(|| format!("{} is not a donor registry: it lists no donors.", registry.display()))?;
    Ok(list.iter().filter_map(|d| Some((d["id"].as_str()?.to_owned(), base.join(d["file"].as_str()?)))).collect())
}

/// Why `exporter` can't build a GDL for the panel `profile`, if it can't: its registry has no
/// prepared donor for it.
pub fn missing_donor(exporter: &Exporter, profile: &str) -> Option<String> {
    let found = donors(&exporter.registry).ok()?.into_iter().find(|(id, _)| id == profile);
    match found {
        Some((_, file)) if file.is_file() => None,
        Some(_) => Some(format!("The donor GDL for {profile} is not prepared. Run the exporter's prepare-donor {profile} first.")),
        None => Some(format!("The donor registry {} has no {profile} donor.", exporter.registry.display())),
    }
}

/// Find the exporter: the environment variables, then the CLI and registry chosen in the app
/// (`settings`), then a sibling `figma-extron-exporter` checkout next to one of `near` or their
/// parents, with the [`EXPORTER_VERSION`] kit and the first registry whose donors are prepared. A
/// variable that is set must be right: it is reported rather than passed over.
pub fn find_exporter(settings: Option<(&Path, &Path)>, near: &[PathBuf]) -> Result<Exporter, String> {
    if let Some((cli_var, cli)) = first_env(&[CLI_ENV, CLI_ENV_OLD]) {
        let (registry_var, registry) = first_env(&[&[REGISTRY_ENV], REGISTRY_ENV_OLD].concat())
            .ok_or_else(|| format!("{cli_var} is set: set {REGISTRY_ENV} to the donor registry too, or unset {cli_var}."))?;
        return check_exporter(&cli, &registry, ExporterSource::Environment)
            .map_err(|e| format!("{e} (from {cli_var} and {registry_var}: correct them, or unset {cli_var} to use the exporter chosen in the app)"));
    }
    if let Some(found) = settings.and_then(|(cli, registry)| check_exporter(cli, registry, ExporterSource::Settings).ok()) {
        return Ok(found);
    }
    for start in near {
        for dir in start.ancestors() {
            let root = dir.join("figma-extron-exporter");
            let kit = root.join("output").join("releases").join(EXPORTER_VERSION);
            let cli = kit.join("smoke-kit").join("cli").join("Extron.Cli.dll");
            if !cli.is_file() {
                continue;
            }
            let registries = [kit.join("smoke-kit").join("samples").join("donors.json"), root.join("samples").join("donors.json")];
            if let Some(found) = registries.iter().find_map(|r| check_exporter(&cli, r, ExporterSource::Sibling).ok()) {
                return Ok(found);
            }
        }
    }
    Err(format!(
        "The Extron exporter {EXPORTER_VERSION} was not found. Choose its Extron.Cli.dll and donors.json (File ▸ Extron ▸ Choose GDL exporter…), or set {CLI_ENV} and {REGISTRY_ENV}."
    ))
}

/// The first of `names` set to a non-empty value, and that value as a path.
fn first_env(names: &[&'static str]) -> Option<(&'static str, PathBuf)> {
    names.iter().find_map(|name| std::env::var_os(name).filter(|v| !v.is_empty()).map(|v| (*name, PathBuf::from(v))))
}

/// The variable naming the exporter CLI ([`CLI_ENV`], or [`CLI_ENV_OLD`]), when one is set:
/// [`find_exporter`] then uses the environment only, whatever the app's settings hold, so choosing
/// an exporter in the app can't fix an error it reports.
pub fn exporter_variable() -> Option<&'static str> {
    first_env(&[CLI_ENV, CLI_ENV_OLD]).map(|(name, _)| name)
}

/// Build `project` as a GDL at `output` with `exporter`, publishing the file only after the
/// exporter reports every independent check passed. `output` must not exist yet.
pub fn build(project: &Project, output: &Path, exporter: &Exporter) -> Result<Value, String> {
    if let Some(problem) = missing_donor(exporter, project.profile.id) {
        return Err(problem);
    }
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
        .arg(&exporter.cli)
        .arg("convert-package")
        .arg(&exui)
        .arg(&gdl)
        .arg("--registry")
        .arg(&exporter.registry)
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{ExporterSource, check_exporter, find_exporter, missing_donor};

    /// A fake exporter checkout under `root`: the 0.8.8 smoke kit's CLI, a smoke-kit registry
    /// without donor files, and the checkout's own registry with the 1025M donor prepared.
    fn checkout(root: &Path) -> (PathBuf, PathBuf) {
        let repo = root.join("figma-extron-exporter");
        let kit = repo.join("output/releases/0.8.8/smoke-kit");
        let registry = |dir: &Path| {
            std::fs::create_dir_all(dir.join("donors")).unwrap();
            let json = r#"{"donors": [{"id": "tlp-pro-1025m", "file": "donors/tlp-pro-1025m.gdl"}, {"id": "tlp-pro-300m-portrait", "file": "donors/tlp-pro-300m-portrait.gdl"}]}"#;
            std::fs::write(dir.join("donors.json"), json).unwrap();
            dir.join("donors.json")
        };
        std::fs::create_dir_all(kit.join("cli")).unwrap();
        std::fs::write(kit.join("cli/Extron.Cli.dll"), b"").unwrap();
        registry(&kit.join("samples"));
        let prepared = registry(&repo.join("samples"));
        std::fs::write(repo.join("samples/donors/tlp-pro-1025m.gdl"), b"").unwrap();
        (kit.join("cli/Extron.Cli.dll"), prepared)
    }

    #[test]
    fn a_sibling_checkout_is_found_with_the_registry_whose_donors_are_prepared() {
        let root = tempfile::tempdir().unwrap();
        let (cli, prepared) = checkout(root.path());
        let app = root.path().join("AV-GUI-Studio/target/release");
        std::fs::create_dir_all(&app).unwrap();
        // The environment of the test run may name an exporter; only the sibling search is checked
        // when it doesn't.
        if super::exporter_variable().is_none() {
            let found = find_exporter(None, &[app]).unwrap();
            assert_eq!((found.cli, found.registry, found.source), (cli.clone(), prepared.clone(), ExporterSource::Sibling));
        }
        let chosen = check_exporter(&cli, &prepared, ExporterSource::Settings).unwrap();
        assert_eq!(missing_donor(&chosen, "tlp-pro-1025m"), None);
        let portrait = missing_donor(&chosen, "tlp-pro-300m-portrait").unwrap();
        assert!(portrait.contains("prepare-donor tlp-pro-300m-portrait"), "{portrait}");
        assert!(missing_donor(&chosen, "tlp-pro-2000").unwrap().contains("has no tlp-pro-2000 donor"));

        let smoke = cli.parent().unwrap().parent().unwrap().join("samples/donors.json");
        let unprepared = check_exporter(&cli, &smoke, ExporterSource::Settings).unwrap_err();
        assert!(unprepared.contains("prepare-donor tlp-pro-1025m"), "{unprepared}");
        let relative = check_exporter(Path::new("Extron.Cli.dll"), &prepared, ExporterSource::Settings).unwrap_err();
        assert!(relative.contains("not a file at an absolute path"), "{relative}");
    }

    #[test]
    fn nothing_found_says_how_to_choose_the_exporter() {
        if super::exporter_variable().is_some() {
            return;
        }
        let empty = tempfile::tempdir().unwrap();
        let none = find_exporter(None, &[empty.path().to_path_buf()]).unwrap_err();
        assert!(none.contains("Choose GDL exporter") && none.contains("AV_GUI_STUDIO_EXTRON_CLI"), "{none}");
    }
}
