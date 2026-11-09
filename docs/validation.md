# Doğrulama — 9 Kasım 2026

## Geçen kontroller

- Apple Silicon macOS üzerinde Node.js 26.7.0, npm 11.19.0, Rust 1.99.1.
- 20 Rust testi: envanter ayrıştırma, özel tap/kaynak ayrımı, belirsiz sürüm reddi, withdrawn / repo / sürüm kanıtı, fixed sınırı, kaynak URL izin listesi, sürüm karşılaştırması, migration ve veri koruma, cache, retention, zamanlayıcı ve retry kuralları.
- 4 frontend testi: boş envanter, kapsam ifadesi, kaynak URL doğrulaması ve dikkat gerektiren sürüm sınıflaması.
- TypeScript kontrolü ve Vite üretim derlemesi geçti.
- Rust fmt ve clippy `-D warnings` geçti.
- Gerçek Homebrew envanteri: 61 paket; geçici SQLite yazma/okuma başarılı.
- Sabit örnek sürümlerle canlı OSV, Node.js üretici, Homebrew API ve Microsoft winget GitHub manifest testleri geçti. Gerçek kurulu envanter bu servis testlerinde gönderilmedi.
- Gerçek Tauri penceresinde envanter, 61 paketlik sonuç, tarama geçmişi, JSON rapor kaydı ve varsayılan kapalı tercihler gözlendi.
- Apple Silicon `.app` ve `.dmg` paketleme başarılı. Son paket yerel ad hoc imzalıdır; Developer ID/notarization değildir.
- `npm audit --audit-level=high`: 0 bildirilen açık.

## RustSec sonucu

Cargo.lock içindeki 493 bağımlılık 1430 kayıt içeren RustSec veritabanıyla denetlendi; komut 0 çıkış koduyla tamamlandı. Beş uyarı raporlandı: proc-macro-error, zbus_names, zvariant, zvariant_derive için bakım durumu; glib için unsoundness bildirimi (RUSTSEC-2024-0429). Bu beş paket Apple Silicon hedefinin seçili bağımlılık grafiğinde bulunmuyor; hedef dışı kilit dosyası bağımlılıklarıdır. Uyarılar bastırılmadı. Diğer platformlar için ayrıca değerlendirme gerekir; bu sonuç bir güvenlik garantisi değildir.

## Henüz doğrulanmayanlar

Windows üzerinde gerçek winget export, NSIS kurulum/kaldırma ve bildirim; macOS Intel çalıştırması; sistem bildiriminin gerçek teslimi; altı saatlik uzun süreli zamanlayıcı oturumu. Bunlar kullanıcı kabul testi ve GitHub CI gerektirir. İş akışları hazırlanmıştır, uzak depoda henüz çalıştırılmamıştır. Genel yayın ve Developer ID/notarization yapılmamıştır.

## 0.2.1 düzeltmeleri

24 Rust ve 5 frontend testi geçti. Yeni testler doğrudan makbuz taraması, Cellar dışına çıkan yollar, olmayan makbuz, bağımlılık ayrımı, eski snapshot gizleme ve uzak IPC origin reddini kapsar. Önceki 0.2.0 GUI gözlemleri yeni sürümün uçtan uca doğrulaması sayılmaz. macOS makbuz doğrulaması envanterdeki 61 kayıt için yerel kayıt buldu; bunların 7’si doğrudan kurulum, 54’ü bağımlılıktır. Bu istatistik yalnızca test çalışma ortamına aittir; kullanıcı her makbuzu kendi cihazında incelemelidir.

0.2.1 gerçek pencere kontrolü: eski kayıt banner’ı, yeni çevrimdışı tarama, 7 doğrudan / 54 bağımlılık ayrımı, varsayılan 7 görünür satır ve Node kurulum makbuzu yolu doğrulandı. Dar IPC izinleriyle tarama ve olay dinleme çalıştı. Özel GitHub güvenlik bildirimleri kullanıcının açık onayıyla açıldı.
