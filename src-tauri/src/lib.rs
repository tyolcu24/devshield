pub mod adapters;
pub mod db;
pub mod model;
pub mod process;
pub mod scheduler;
pub mod security;
use adapters::InventoryAdapter;
use model::{History, Scan, Settings};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
struct ScanState(Arc<AtomicBool>);
struct Guard(Arc<AtomicBool>);
impl Drop for Guard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
fn trusted_origin(label: &str, url: &tauri::Url) -> bool {
    if label != "main" {
        return false;
    }
    let bundled = (url.scheme() == "tauri" && url.host_str() == Some("localhost"))
        || (url.scheme() == "http"
            && url.host_str() == Some("tauri.localhost")
            && url.port().is_none());
    let development = cfg!(debug_assertions)
        && url.scheme() == "http"
        && [Some("localhost"), Some("127.0.0.1")].contains(&url.host_str())
        && url.port() == Some(1420);
    bundled || development
}
fn authorize(window: &tauri::WebviewWindow) -> Result<(), String> {
    let url = window.url().map_err(|e| e.to_string())?;
    if trusted_origin(window.label(), &url) {
        Ok(())
    } else {
        Err("Bu pencereye yerel veri erişimi verilmedi".into())
    }
}
fn database(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    Ok(dir.join("devshield.sqlite"))
}
fn perform(app: &tauri::AppHandle) -> Result<Scan, String> {
    let state = app.state::<ScanState>();
    if state
        .0
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err("Tarama zaten çalışıyor".into());
    }
    let _guard = Guard(state.0.clone());
    let _ = app.emit("scan-progress", "Yerel paket envanteri okunuyor…");
    let path = database(app)?;
    let settings = db::settings(&path)?;
    let adapter: Box<dyn InventoryAdapter> = if cfg!(target_os = "macos") {
        Box::new(adapters::Homebrew)
    } else {
        Box::new(adapters::Winget)
    };
    let mut packages = adapter.discover()?;
    let mut warnings = vec![];
    if packages.is_empty() {
        warnings.push("Envanter boş. Bu sonuç cihazın güvenli olduğunu göstermez.".into());
    }
    warnings.push(if cfg!(windows){"winget yalnızca kaynakla eşleştirebildiği uygulamaları dışa aktarır; envanter tam cihaz taraması değildir."}else{"Homebrew Cellar/Caskroom kurulum makbuzları okunur. Diğer uygulamalar ve proje bağımlılık dosyaları taranmaz. Güncel sürüm sorgusu yalnızca çevrimiçi tercih açılırsa yapılır."}.into());
    if settings.online {
        let sources = security::Sources::new()?;
        for p in &mut packages {
            if p.manager.starts_with("homebrew") && p.verification != "receipt_found" {
                p.security_status = "unverified_inventory".into();
                continue;
            }
            if security::identity(p).is_none() {
                p.security_status = "unsupported_identity".into();
                continue;
            }
            let _ = app.emit(
                "scan-progress",
                format!("Güvenlik kaynağı kontrol ediliyor: {}", p.id),
            );
            if let Err(e) = sources.check(&path, p) {
                p.security_status = "source_error".into();
                warnings.push(format!("{}: {e}", p.id));
            }
        }
        for p in &mut packages {
            let _ = app.emit(
                "scan-progress",
                format!("Sürüm kaynağı kontrol ediliyor: {}", p.id),
            );
            if let Err(e) = sources.update(&path, p) {
                warnings.push(format!("{} sürüm kaynağı: {e}", p.id));
            }
        }
    }
    let scan = Scan {
        packages,
        warnings,
        checked_at: model::now(),
        online: settings.online,
    };
    db::save(&path, &scan)?;
    let count: usize = scan.packages.iter().map(|p| p.findings.len()).sum();
    let updates = scan
        .packages
        .iter()
        .filter(|p| p.update_status == "security_update_available")
        .count();
    if settings.notifications && (count > 0 || updates > 0) {
        if let Err(e)=app.notification().builder().title("DevShield taraması tamamlandı").body(format!("{count} incelenmesi gereken güvenlik kaydı, {updates} güvenlik sürümü bildirimi. Ayrıntılar için uygulamayı açın.")).show(){let _=app.emit("scan-notice",format!("Sistem bildirimi gönderilemedi: {e}"));}
    }
    let _ = app.emit("scan-completed", &scan);
    Ok(scan)
}
#[tauri::command]
async fn scan_inventory(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<Scan, String> {
    authorize(&window)?;
    tauri::async_runtime::spawn_blocking(move || perform(&app))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
fn get_settings(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<Settings, String> {
    authorize(&window)?;
    db::settings(&database(&app)?)
}
#[tauri::command]
fn save_settings(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    settings: Settings,
) -> Result<(), String> {
    authorize(&window)?;
    if settings.notifications {
        let permission = app
            .notification()
            .request_permission()
            .map_err(|e| e.to_string())?;
        if permission != tauri::plugin::PermissionState::Granted {
            return Err("Bildirim izni verilmedi. Sistem ayarlarından izin verebilirsiniz.".into());
        }
    }
    db::set_settings(&database(&app)?, &settings)
}
#[tauri::command]
fn latest_scan(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<Option<Scan>, String> {
    authorize(&window)?;
    db::latest(&database(&app)?)
}
#[tauri::command]
fn scan_history(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<Vec<History>, String> {
    authorize(&window)?;
    db::history(&database(&app)?)
}
#[tauri::command]
fn export_report(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<String, String> {
    authorize(&window)?;
    let path = database(&app)?;
    let scan = db::latest(&path)?.ok_or("Dışa aktarılacak tarama bulunamadı")?;
    // Fixed application-private output path, no frontend path input or overwrite of arbitrary files.
    let output = path.with_file_name("devshield-report.json");
    let mut file = tempfile::NamedTempFile::new_in(path.parent().ok_or("Veri dizini eksik")?)
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    file.write_all(&serde_json::to_vec_pretty(&scan).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(&output).map_err(|e| e.to_string())?;
    Ok(output.to_string_lossy().into())
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(ScanState(Arc::new(AtomicBool::new(false))))
        .setup(|app| {
            let handle = app.handle().clone();
            let path = database(&handle).map_err(std::io::Error::other)?;
            let last = db::latest(&path)
                .map_err(std::io::Error::other)?
                .map(|s| s.checked_at);
            std::thread::spawn(move || {
                let mut schedule = scheduler::Schedule::new(last);
                let mut last_seen = last;
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(15));
                    let latest_time = db::latest(&path).ok().flatten().map(|s| s.checked_at);
                    if latest_time != last_seen {
                        schedule = scheduler::Schedule::new(latest_time);
                        last_seen = latest_time;
                    }
                    match db::settings(&path) {
                        Ok(settings) if settings.periodic => {
                            // Wall-clock overdue check catches wake from sleep; monotonic due handles retries.
                            let overdue = schedule.failures == 0
                                && db::latest(&path).ok().flatten().is_none_or(|s| {
                                    model::now().saturating_sub(s.checked_at)
                                        >= scheduler::INTERVAL + 120
                                });
                            if std::time::Instant::now() >= schedule.due || overdue {
                                let result = perform(&handle);
                                if let Err(e) = &result {
                                    let _ = handle
                                        .emit("scan-notice", format!("Periyodik tarama: {e}"));
                                }
                                schedule.finished(result.is_ok());
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            let _ = handle.emit("scan-notice", format!("Ayarlar okunamadı: {e}"));
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_inventory,
            get_settings,
            save_settings,
            latest_scan,
            scan_history,
            export_report
        ])
        .run(tauri::generate_context!())
        .expect("DevShield başlatılamadı");
}

#[cfg(test)]
mod ipc_tests {
    use super::*;
    #[test]
    fn remote_origins_and_other_windows_are_denied() {
        assert!(!trusted_origin(
            "main",
            &tauri::Url::parse("https://evil.test").unwrap()
        ));
        assert!(!trusted_origin(
            "other",
            &tauri::Url::parse("tauri://localhost").unwrap()
        ));
        assert!(!trusted_origin(
            "main",
            &tauri::Url::parse("http://tauri.localhost.evil.test").unwrap()
        ));
        assert!(trusted_origin(
            "main",
            &tauri::Url::parse("tauri://localhost").unwrap()
        ));
    }
}
