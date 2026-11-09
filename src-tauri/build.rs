fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "scan_inventory",
            "get_settings",
            "save_settings",
            "latest_scan",
            "scan_history",
            "export_report",
        ]),
    ))
    .expect("DevShield izinleri oluşturulamadı");
}
