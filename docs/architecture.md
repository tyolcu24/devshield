# Mimari

React → dar Tauri IPC → tarama kilidi → platform envanter adaptörü → kaynak adaptörleri → SQLite işleminde tarama kaydı → arayüz olayı / bildirim.

## Güven sınırları

Frontend yürütülebilir dosya, argüman, SQL veya kaynak URL’si seçemez. macOS adaptörü Homebrew komutu çalıştırmaz: standart Cellar/Caskroom dizinleri ve 512 KiB sınırındaki makbuzlar doğrudan okunur. Kanonik yolların beklenen dizin içinde kaldığı kontrol edilir. Makbuzlar kurulum kaydıdır, binary imzası/bütünlüğü kanıtı değildir. Doğrudan kurulum/bağımlılık makbuz alanından alınır. Özel prefix ve kayıtsız cask’ler kapsam dışında kalabilir. Windows’ta Microsoft App Installer’ın kullanıcıya ait WindowsApps winget alias’ı sabit argümanlarla export için çalıştırılır. PATH araması, shell ve PowerShell yoktur. winget kaynak anlaşmaları kendiliğinden kabul edilmez. Export yalnızca uygulamaya ait geçici dizine yazılır ve temizlenir. Çıktı 8 MiB, süreç 45 saniye ile sınırlıdır.

IPC için build manifest komut izinleri açıkça üretilir; yalnızca ana yerel pencereye atanır. Ayrıca her komut pencere etiketi ve URL origin kontrolü yapar. CSP uzak sayfa, iframe, form, object ve harici script yüklemeyi engeller. Kaynak metni React tarafından düz metin olarak gösterilir; HTML/komut olarak yürütülmez. AI/LLM, uzaktan dinleyen servis ve dosya yükleme yoktur. Unix uygulama veri dizini 0700 ve DB 0600 tutulur. DB symlink’i reddedilir; rapor geçici özel dosyaya yazılıp atomik olarak değiştirilir. Aynı kullanıcı hesabını ele geçirmiş başka yazılımlar ve işletim sistemi/WebView açıkları bu sınırların dışında kalır.

Ağ sadece dört sabit HTTPS kaynağına gider: OSV, Node.js, Homebrew API ve Microsoft winget GitHub manifestleri. URL yolları katı karakter kurallarıyla üretilir; yönlendirmeler izlenmez, TLS doğrulaması kapatılmaz. Yanıt sınırı 4 MiB; istek 10 saniye, bağlantı 4 saniyedir. Tarama başına 60 ağ isteği ve yaklaşık 120 saniye bütçe vardır; başlamış istek son süreyi en fazla bir istek timeout’u kadar aşabilir. Önce güvenlik sorguları, sonra sürüm sorguları yapılır. Rate-limit / hata sonuçları kullanıcıya gösterilir; kaynakta temiz sonuç gibi saklanmaz. Bozuk yanıtlar kanıt sayılmaz. Önbellek tazeliği bir saattir; eski cache çevrimdışı temiz sonuç üretmek için kullanılmaz.

## Yanlış pozitif politikası

Envanter kimliği npm paket adı değildir. Homebrew core tap’i ve Microsoft winget resmi source identifier’ı kontrol edilir; yalnızca kodda incelenebilir açık eşleştirmeler upstream repo kimliğine çevrilir. Özel tap veya kaynakların benzer adları güvenlik sorgusuna alınmaz. Sürümler tam numeric semver olmalı; revizyon, build ve prerelease otomatik sıyrılmaz. OSV GIT/tag sorgusu bulanık eşleşme yapabildiğinden dönen kayıt yeniden kontrol edilir.

OSV affected kaydı aynı repo kimliğini taşımalı; tam sürüm etiketi `versions` içinde bulunmalı veya aynı GIT paketinin SEMVER aralığı sürümü kapsamalıdır. GIT commit aralıkları sürüm etiketine ait ek kanıt olmadan kesin kabul edilmez. Withdrawn kayıtlar atılır. Açık uçlu/limit/fixed aralıkları ayrı işlenir; fixed sınırı hariçtir. Sonuçlar OSV id’siyle tekilleştirilir. Aynı CVE’nin farklı advisory id’leri ayrı görünebilir; alias birleştirmesi yapılmaz. Tüm eşleşmeler upstream inceleme adayıdır: dağıtıcı backport/yamaları doğrulanmadığı için cihazın kesin savunmasız olduğu iddia edilmez. OSV’de kayıt olmaması kapsam yokluğu da olabilir.

Node üretici adaptörü aynı ana sürüm dalındaki sürümleri ve aradaki `security` sürüm işaretlerini okur; bu yalnızca güvenlik sürümü bildirimi olup belirli bir CVE kanıtı değildir. Genel sürüm karşılaştırıcısı aynı sayıda 2–4 numeric bileşen ve Homebrew revizyonu için sıralama yapar. Karma sürümler, prerelease ve farklı bileşen sayıları elle incelemeye bırakılır. Yükseltme komutu üretilmez/çalıştırılmaz.

## Veri ve zamanlama

SQLite v1 envanter tabloları korunur; v2 migration raporlar, tercihler ve kaynak önbelleği ekler. Daha yeni schema eski uygulamayla açılmaz. Tarama ve paketler atomik transaction’da kaydedilir; son 100 rapor korunur. `scan_reports` güvenlik kanıtı dahil tam taramayı saklar; v1 identities/findings tabloları geriye uyumluluk için korunur, yeni arayüz raporları okur. Cache bir gün sonra yazma sırasında temizlenir. Veritabanı erişiminde 5 saniye kilit bekleme sınırı vardır.

Periyodik kontrol varsayılan kapalıdır. Uygulama açıkken 15 saniyelik denetim döngüsü, yaklaşık 6 saat interval ve 0–119 saniye jitter kullanılır. Monoton saat ve kalıcı son tarama zamanı birlikte uyku/uyanma ve saat geri alınmasını ele alır. Elle ve zamanlanmış tarama aynı atomik kilidi kullanır. Başarılı elle tarama zamanlayıcıyı ileri taşır; hata 1 dakikadan başlayıp en çok 1 saate çıkan geri çekilmeye girer. İşlem kuyruğu birikmez. Uygulama kapalıyken ayrı servis veya başlangıç kaydı yoktur.

Bildirimler yalnızca kullanıcı açarsa işletim sistemi izni istenerek gönderilir. Kayıt veya yeni Node güvenlik sürümü olduğunda özet bildirilir; her taramada aynı kayıtlara tekrar bildirim gelebilir. Kaynak hataları uygulama içinde görünür.

## Sürüm sınırları

Bu sürüm kapsamlı AV, SBOM veya SCA ürünü değildir. Commit aralıklarından etiket çözümleme, üretici backport doğrulaması, CVE alias birleştirme, uygulama kapalıyken servis ve imzalı güncelleyici yoktur. Otomatik güncelleyici bilinçli olarak bulunmaz. Windows gerçek cihaz çalıştırması ve sistem bildirimi ayrı kabul testidir.
