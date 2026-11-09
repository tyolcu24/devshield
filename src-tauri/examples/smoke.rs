//! Explicit local / live-source integration probe. Never installs or updates software.
use devshield_lib::{
    adapters::{Homebrew, InventoryAdapter, Winget},
    db,
    model::{now, Package, Scan},
    security::Sources,
};
fn main() -> Result<(), String> {
    let adapter: Box<dyn InventoryAdapter> = if cfg!(target_os = "macos") {
        Box::new(Homebrew)
    } else {
        Box::new(Winget)
    };
    let packages = adapter.discover()?;
    if packages.is_empty() {
        return Err("Envanter boş; entegrasyon doğrulanamadı".into());
    }
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let path = dir.path().join("probe.sqlite");
    let scan = Scan {
        packages,
        warnings: vec![],
        checked_at: now(),
        online: false,
    };
    db::save(&path, &scan)?;
    let loaded = db::latest(&path)?.ok_or("Kaydedilmiş tarama okunamadı")?;
    assert_eq!(loaded.packages.len(), scan.packages.len());
    println!("PASS inventory + SQLite round-trip: {} packages; {} explicitly requested, {} dependencies, {} receipt-backed",loaded.packages.len(),loaded.packages.iter().filter(|p|p.install_reason=="requested").count(),loaded.packages.iter().filter(|p|p.install_reason=="dependency").count(),loaded.packages.iter().filter(|p|p.verification=="receipt_found").count());
    if std::env::args().any(|a| a == "--online") {
        let sources = Sources::new()?;
        let mut node = Package::new("homebrew", "node", "22.1.0");
        sources.check(&path, &mut node)?;
        sources.update(&path, &mut node)?;
        if node.latest.is_none() {
            return Err("Node üretici sürümü bulunamadı".into());
        }
        println!(
            "PASS live OSV + Node vendor: security={}, update={}",
            node.security_status, node.update_status
        );
        let mut curl = Package::new("homebrew", "curl", "8.5.0");
        sources.update(&path, &mut curl)?;
        if curl.latest.is_none() {
            return Err("Homebrew sürümü bulunamadı".into());
        }
        println!("PASS live Homebrew update API");
        let mut windows_node = Package::new("winget", "Git.Git", "2.40.0");
        sources.update(&path, &mut windows_node)?;
        if windows_node.latest.is_none() {
            return Err("winget manifest sürümü bulunamadı".into());
        }
        println!("PASS live winget manifest API (not Windows runtime)");
    }
    Ok(())
}
