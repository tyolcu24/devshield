//! No shell, PATH search, user-supplied executable or arbitrary IPC arguments.
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
const LIMIT: usize = 8 * 1024 * 1024;
pub fn run(executable: &Path, args: &[&std::ffi::OsStr]) -> Result<Vec<u8>, String> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .env("HOMEBREW_NO_AUTO_UPDATE", "1")
        .env("HOMEBREW_NO_ANALYTICS", "1")
        .env("HOMEBREW_NO_ENV_HINTS", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("Envanter işlemi başlatılamadı: {e}"))?;
    let stdout = child.stdout.take().ok_or("stdout eksik")?;
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut bytes = vec![];
        let result = stdout
            .take((LIMIT + 1) as u64)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = sender.send(result);
    });
    let start = Instant::now();
    let mut output = None;
    loop {
        if let Ok(result) = receiver.try_recv() {
            let bytes = result.map_err(|e| e.to_string())?;
            if bytes.len() > LIMIT {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Envanter çıktı sınırı aşıldı".into());
            }
            output = Some(bytes);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(format!("Envanter komutu başarısız: {status}. Paket yöneticisini ve kaynak anlaşmalarını kontrol edin."));
                }
                if let Some(bytes) = output {
                    return Ok(bytes);
                }
                return receiver
                    .recv_timeout(Duration::from_secs(1))
                    .map_err(|_| "Envanter çıktısı tamamlanamadı".to_string())?
                    .map_err(|e| e.to_string())
                    .and_then(|b| {
                        if b.len() > LIMIT {
                            Err("Envanter çıktı sınırı aşıldı".into())
                        } else {
                            Ok(b)
                        }
                    });
            }
            Ok(None) if start.elapsed() < Duration::from_secs(45) => {
                std::thread::sleep(Duration::from_millis(30))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Envanter işlemi 45 saniyede tamamlanamadı".into());
            }
        }
    }
}
pub fn text(executable: &Path, args: &[&str]) -> Result<String, String> {
    let args: Vec<_> = args.iter().map(std::ffi::OsStr::new).collect();
    String::from_utf8(run(executable, &args)?).map_err(|e| e.to_string())
}
