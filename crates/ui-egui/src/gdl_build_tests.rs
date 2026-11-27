//! File ▸ Extron ▸ Build GDL… runs the exporter on a worker thread: a banner says so, the Extron
//! export actions wait for it, and the result arrives as a toast while the window keeps working.

use figcraft_engine::Session;
use serde_json::json;

use crate::layers_drag_tests::Ui;

/// An exporter that can't build: an empty `Extron.Cli.dll`, so `dotnet` (or its absence) fails,
/// with the 1025M donor in place so the build gets as far as running it.
fn broken_exporter(dir: &std::path::Path) -> (String, String) {
    std::fs::create_dir_all(dir.join("donors")).unwrap();
    std::fs::write(dir.join("Extron.Cli.dll"), b"").unwrap();
    std::fs::write(dir.join("donors/tlp-pro-1025m.gdl"), b"").unwrap();
    std::fs::write(dir.join("donors.json"), r#"{"donors": [{"id": "tlp-pro-1025m", "file": "donors/tlp-pro-1025m.gdl"}]}"#).unwrap();
    (dir.join("Extron.Cli.dll").to_string_lossy().into_owned(), dir.join("donors.json").to_string_lossy().into_owned())
}

#[test]
fn a_gdl_builds_on_a_worker_thread_and_its_result_arrives_as_a_toast() {
    // A set environment variable would pick another exporter.
    if std::env::var_os(figcraft_engine::gui_studio_extron::CLI_ENV).is_some() {
        return;
    }
    let dir = std::env::temp_dir().join(format!("gui-studio-gdl-thread-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (cli, registry) = broken_exporter(&dir);
    let mut s = Session::new();
    s.execute("extron.demo", &json!({})).unwrap();
    s.execute("extron.setExporter", &json!({"cli": cli, "registry": registry})).unwrap();
    let mut u = Ui::new(s);
    u.frame(vec![]);

    crate::extron::start_build(&mut u.app, dir.join("room.gdl"));
    assert_eq!(u.app.ui.gdl_build.as_ref().map(|b| b.name.as_str()), Some("room.gdl"), "the build runs on its own thread");
    // Meanwhile the Extron exports wait.
    crate::extron::save(&mut u.app, "extron.exportExui");
    assert_eq!(u.app.ui.toast.as_ref().map(|(m, _)| m.as_str()), Some("A GDL is being built. Wait for it to finish."));

    let start = std::time::Instant::now();
    while u.app.ui.gdl_build.is_some() && start.elapsed() < std::time::Duration::from_secs(60) {
        u.frame(vec![]);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(u.app.ui.gdl_build.is_none(), "the build finished");
    let toast = u.app.ui.toast.as_ref().map(|(m, _)| m.clone()).unwrap_or_default();
    assert!(toast.starts_with("GDL build failed."), "{toast}");
    assert!(!dir.join("room.gdl").exists(), "a failed build writes no GDL");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn where_files_are_downloads_export_exui_downloads_the_package() {
    use crate::variables_export_ui_tests::{Browser, Downloads};
    let mut s = Session::new();
    s.execute("extron.demo", &json!({})).unwrap();
    let got = Downloads::default();
    let mut app = crate::FigApp::new(s, Box::new(Browser { got: got.clone(), deliver: true }));
    crate::extron::save(&mut app, "extron.exportExui");
    let got = got.lock().unwrap();
    let [(name, bytes)] = got.as_slice() else { panic!("one download, got {}", got.len()) };
    assert!(name.ends_with(".exui"), "{name}");
    let project = figcraft_engine::gui_studio_extron::project(app.session.doc().unwrap()).unwrap();
    assert_eq!(bytes, &figcraft_engine::gui_studio_extron::package(&project).unwrap());
    assert_eq!(app.ui.toast.as_ref().map(|(m, _)| m.as_str()), Some("EXUI exported."));
}
