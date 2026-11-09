# DevShield

macOS ve Windows için açık kaynak geliştirici yazılım izleme uygulaması. Tauri 2 + Rust, React + TypeScript ve yerel SQLite. **0.2.1 test adayıdır; kaynak kodu herkese açıktır; deneysel test sürümüdür.**

## Neler yapar?

- macOS’ta Homebrew Cellar/Caskroom kurulum makbuzlarından formula ve cask envanteri; Windows’ta winget’in resmi kaynağa eşleyebildiği uygulamaların JSON envanteri.
- macOS kurulu sürümünü yerel makbuz/dizinlerden, Windows sürümünü winget export kaydından alır; taranan paketlerin binary’lerini çalıştırmaz. Doğrudan kurulum ve bağımlılık ayrımı ile makbuz yolunu gösterir.
- Çevrimdışı taramada yalnızca yerel kurulum verisini gösterir. İsteğe bağlı çevrimiçi modda Homebrew API, Microsoft winget manifestleri ve Node.js üretici sürümlerini kontrol eder.
- OSV üzerinde yalnızca sınırlı, açıkça tanımlanmış upstream kimlikleri sorgular. Kaynak kanıtı yetersizse sonuç belirsiz kalır. Upstream kayıtları üretici backport’ları doğrulanana kadar **inceleme adayıdır**, kesin cihaz açığı değildir.
- Elle veya uygulama açıkken yaklaşık 6 saatte bir tarama; çakışma engeli, hata geri çekilmesi, isteğe bağlı sistem bildirimleri.
- Son 100 tarama, kalıcı tercihler, bir saat kaynak önbelleği ve yerel JSON raporu.

Yazılım yüklemez/yükseltmez. Shell, PowerShell veya kullanıcı komutu çalıştırmaz. Ağ kontrolleri, periyodik tarama ve bildirimler ilk açılışta kapalıdır.

## Test paketini kullanma

Yerel teslimler `artifacts/` klasörüne konur. Apple Silicon Mac için `.dmg` dosyasını açıp uygulamayı Applications klasörüne sürükleyin veya `.app.zip` arşivini açın. İlk açılışta **Şimdi tara** düğmesine basın. Çevrimiçi kontrolü açmadan önce Tercihler ekranındaki veri açıklamasını okuyun. [Test rehberi](docs/testing.md).

Bu yerel paket Apple Developer kimliğiyle imzalanıp noter onayından geçirilmedi. macOS güvenlik uyarısı verebilir; korumaları otomatik değiştiren bir komut verilmez. Genel dağıtım için [yayın koşulları](docs/releasing.md) tamamlanmalıdır. Windows kurucusu bu Mac üzerinde oluşturulamaz; `Build release candidates` GitHub iş akışı Windows `.exe` ve evrensel macOS `.dmg` üretmek için hazırlanmıştır, henüz çalıştırılmamıştır.

## Kaynaktan çalıştırma

Node.js 22.12+ (tercihen güncel LTS), npm ve güncel kararlı Rust/Cargo gerekir. macOS: Xcode Command Line Tools. Windows: MSVC C++ Build Tools, Windows SDK, WebView2 ve Microsoft App Installer (winget). [Tauri önkoşulları](https://v2.tauri.app/start/prerequisites/).

```sh
npm ci
npm run tauri dev
```

Yalnızca arayüz önizlemesi için `npm run dev`; tarayıcıda gerçek tarama devre dışıdır.

```sh
npm test
npm run build
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo audit --file src-tauri/Cargo.lock
```

Paketleme:

```sh
# macOS: derleyen Mac'in mimarisi
npm run tauri build -- --bundles app,dmg
# Windows üzerinde (Windows yapılandırması otomatik birleşir)
npm run tauri build -- --bundles nsis
```

Yerel Rust araçları bu çalışma ortamında `.tools/` içine kurulmuştur. Kalıcı PATH veya kullanıcı kabuk ayarları değiştirilmedi. Normal Rust kurulumu varsa bu özel klasöre gerek yoktur.

## Kapsam ve gizlilik

Tam cihaz, kötü amaçlı yazılım, işletim sistemi veya proje lockfile tarayıcısı değildir. winget, kaynağa eşlenemeyen programları dışarıda bırakabilir. Özel tap/kaynaklar görünür ancak resmi güvenlik eşleştirmesine girmez. Bilinmeyen sürüm, ağ hatası veya sıfır bulgu güvenli kabul edilmez.

Güvenlik kimlikleri şu an Homebrew core için Node.js, curl, git, Go, Rust, Python 3.12/3.13/3.14; resmi winget için OpenJS.NodeJS, OpenJS.NodeJS.LTS, Git.Git, GoLang.Go, Rustlang.Rust.MSVC ile sınırlıdır. `_1` gibi paket revizyonları veya belirsiz sürüm biçimleri güvenlik sorgusuna girmez. Bu liste genişletilebilir; otomatik ad benzerliği kullanılmaz. Kaynak kapsamı bir güvenlik puanı değildir.

Çevrimiçi mod OSV’ye desteklenen upstream repo kimliğini ve sürüm etiketini gönderir. Sürüm kontrolü paket kimlikleriyle `formulae.brew.sh` ve Microsoft’un `api.github.com` deposunu, Node sürümleri için `nodejs.org` adresini kullanır. IP adresi servislerce görülebilir ve sorgular ilişkilendirilebilir. Dosya içerikleri, kullanıcı adı veya tüm envanter tek yük olarak gönderilmez. Telemetri yoktur. Taramada en fazla 60 ağ isteği, toplam kaynak kontrolünde yaklaşık 120 saniyelik bütçe ve istek başına 10 saniye sınırı vardır. Kota hataları görünür kalır; kaynak sonuçları bir saat önbelleğe alınır.

Veriler Tauri uygulama veri dizinindeki `devshield.sqlite` dosyasında, JSON raporu aynı dizinde saklanır; şifreleme yoktur. Örneğin macOS’ta `~/Library/Application Support/io.devshield.desktop/`. Uygulama rapor dışa aktarımında yalnızca kendi sabit rapor dosyasını yazar. Periyodik kontrol uygulama kapalıyken çalışmaz. Tercihlerde yapılan değişiklikler sonraki taramayı etkiler; devam eden taramayı iptal etmez.

[Mimari](docs/architecture.md) · [Doğrulama](docs/validation.md) · [Güvenlik bildirimi](SECURITY.md) · [Değişiklikler](CHANGELOG.md)

MIT lisansı.

## 0.2.1 doğruluk ve erişim sınırları

macOS envanterinde Homebrew komutu çalıştırılmaz; yalnızca standart `/opt/homebrew` ve `/usr/local` Cellar/Caskroom dizinleri ile boyut sınırlı kurulum makbuzları okunur. Bu makbuzlar gerçek çalıştırılabilir dosyanın bütünlük veya imza garantisi değildir. Özel Homebrew prefix’i otomatik aranmaz. Bağımlılıklar varsayılan gizlidir; istenirse ayrıca gösterilir. Her satırda doğrudan kurulum/bağımlılık ve tespit kanıtı vardır. Önceki tarama güncel cihaz durumu olarak sunulmaz; eski kanıtsız kayıtlar varsayılan gizlidir.

AI/LLM entegrasyonu, uzaktan komut sunucusu, genel dosya tarama veya dosya yükleme özelliği yoktur. Uygulama yerel kullanıcı yetkileriyle çalışır; aynı kullanıcı hesabını ele geçirmiş bir yazılıma veya işletim sistemi açığına karşı tam izolasyon sağlamaz. macOS uygulama veri dizini 0700, veritabanı 0600 izinleriyle tutulur. IPC komutları yalnızca ana yerel pencereye açılır; uzak içerik/iframe yüklenmesi kısıtlanır. Bu önlemler güvenlik garantisi veya bağımsız penetrasyon testi yerine geçmez.
