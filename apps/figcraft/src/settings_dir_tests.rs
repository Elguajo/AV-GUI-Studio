//! Where the settings live (#1320): a portable copy keeps them next to the executable, taking the
//! ones it had before; without a marker they stay in the platform folder they always used. Every
//! folder here is a temporary one: the probe sees no real environment, executable or profile.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use craft_settings::Probe;
use figcraft_engine::Prefs;

use crate::services::{Desktop, settings_dir, write_prefs};

/// A fresh temporary folder with an `app` folder (the executable's) and an `appdata` folder (the
/// platform's config folder).
fn root(name: &str) -> PathBuf {
    let r = std::env::temp_dir().join(format!("figcraft-settings-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r);
    std::fs::create_dir_all(r.join("app")).unwrap();
    std::fs::create_dir_all(r.join("appdata")).unwrap();
    r
}

/// What the rules see: the executable in `root/app`, the platform folder `platform`, and
/// `AV_GUI_STUDIO_CONFIG_DIR` set to `config_dir` when given.
fn probe(root: &Path, platform: Option<PathBuf>, config_dir: Option<&'static str>) -> Probe {
    probe_with(root, platform, config_dir.map(|dir| ("AV_GUI_STUDIO_CONFIG_DIR", dir)).into_iter().collect())
}

/// [`probe`] with the environment variables `vars`.
fn probe_with(root: &Path, platform: Option<PathBuf>, vars: Vec<(&'static str, &'static str)>) -> Probe {
    Probe {
        env: Box::new(move |name| vars.iter().find(|(var, _)| *var == name).map(|(_, value)| OsString::from(value))),
        exe_dir: Some(root.join("app")),
        platform_config_dir: platform,
    }
}

/// Preferences with a name to recognise them by.
fn prefs(name: &str) -> Prefs {
    Prefs { user_name: name.into(), ..Prefs::default() }
}

/// The user name of the preferences the app starts with from `dir`.
fn loaded_name(dir: PathBuf) -> Option<String> {
    Desktop::load_from(Some(dir), Arc::new(Mutex::new(None))).prefs.map(|p| p.user_name)
}

#[test]
fn without_a_marker_the_settings_stay_where_they_always_were() {
    let root = root("installed");
    let old = root.join("appdata").join("av-gui-studio");
    write_prefs(&old, &prefs("Ada"));
    let dir = settings_dir(&probe(&root, Some(root.join("appdata")), None));
    assert_eq!(dir, Some(old.clone()), "the installed AV GUI Studio profile folder");
    assert_eq!(loaded_name(old), Some("Ada".into()), "an existing user's preferences are read");
    assert!(!root.join("app").join("config").exists(), "nothing is written next to the executable");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_portable_marker_keeps_the_settings_next_to_the_executable_and_takes_the_old_ones() {
    let root = root("portable");
    let old = root.join("appdata").join("av-gui-studio");
    write_prefs(&old, &prefs("Ada"));
    std::fs::write(root.join("app").join("portable"), "").unwrap();
    let portable = root.join("app").join("config");
    let dir = settings_dir(&probe(&root, Some(root.join("appdata")), None));
    assert_eq!(dir, Some(portable.clone()));
    assert_eq!(loaded_name(portable.clone()), Some("Ada".into()), "the copy starts with the settings it had");
    assert_eq!(loaded_name(old.clone()), Some("Ada".into()), "the installed app's stay where they were");
    // Later starts keep the portable copy's own settings.
    write_prefs(&portable, &prefs("Grace"));
    assert_eq!(settings_dir(&probe(&root, Some(root.join("appdata")), None)), Some(portable.clone()));
    assert_eq!(loaded_name(portable), Some("Grace".into()), "never overwritten by the old ones again");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_new_portable_copy_starts_without_settings() {
    let root = root("fresh");
    std::fs::write(root.join("app").join("portable"), "").unwrap();
    let portable = root.join("app").join("config");
    assert_eq!(settings_dir(&probe(&root, Some(root.join("appdata")), None)), Some(portable.clone()));
    assert_eq!(loaded_name(portable), None, "a first run");
    // No platform folder at all: the marker still decides.
    assert_eq!(settings_dir(&probe(&root, None, None)), Some(root.join("app").join("config")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_config_dir_variable_names_the_folder_and_empty_means_none() {
    let root = root("env");
    std::fs::write(root.join("app").join("portable"), "").unwrap();
    let named = settings_dir(&probe(&root, Some(root.join("appdata")), Some("elsewhere")));
    assert_eq!(named, Some(PathBuf::from("elsewhere")), "ahead of the marker");
    assert_eq!(settings_dir(&probe(&root, Some(root.join("appdata")), Some(""))), None, "isolated: nothing read or written");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn without_a_platform_folder_there_are_no_settings() {
    let root = root("nowhere");
    assert_eq!(settings_dir(&probe(&root, None, None)), None, "not the current directory");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_folder_from_before_the_rename_moves_to_the_new_name() {
    let root = root("renamed");
    let (old, new) = (root.join("appdata").join("gui-studio"), root.join("appdata").join("av-gui-studio"));
    write_prefs(&old, &prefs("Ada"));
    std::fs::create_dir_all(old.join("Plugins")).unwrap();
    let dir = settings_dir(&probe(&root, Some(root.join("appdata")), None));
    assert_eq!(dir, Some(new.clone()));
    assert_eq!(loaded_name(new.clone()), Some("Ada".into()), "the preferences came along");
    assert!(new.join("Plugins").is_dir() && !old.exists(), "the whole folder moved");
    // Once the new folder exists, an old one that turns up again is left alone.
    write_prefs(&old, &prefs("Grace"));
    assert_eq!(settings_dir(&probe(&root, Some(root.join("appdata")), None)), Some(new.clone()));
    assert_eq!(loaded_name(new), Some("Ada".into()));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_variable_from_before_the_rename_counts_when_the_new_one_is_unset() {
    let root = root("old-env");
    let appdata = Some(root.join("appdata"));
    let old_only = probe_with(&root, appdata.clone(), vec![("GUI_STUDIO_CONFIG_DIR", "old")]);
    assert_eq!(settings_dir(&old_only), Some(PathBuf::from("old")));
    assert_eq!(settings_dir(&probe_with(&root, appdata.clone(), vec![("GUI_STUDIO_CONFIG_DIR", "")])), None, "isolated");
    let both = probe_with(&root, appdata, vec![("GUI_STUDIO_CONFIG_DIR", "old"), ("AV_GUI_STUDIO_CONFIG_DIR", "new")]);
    assert_eq!(settings_dir(&both), Some(PathBuf::from("new")), "the new name wins");
    let _ = std::fs::remove_dir_all(&root);
}
