# Yayın akışı

0.2.0 yerel test adayıdır. Kullanıcı testinden sonra birlikte genel yayına geçilir; bu çalışma sırasında uzak depo veya herkese açık release oluşturulmadı.

## Yayın öncesi gerekenler

- macOS ve Windows CI yeşil olmalı. `Build release candidates` elle başlatılarak macOS universal DMG ve Windows x64 NSIS kurucusu alınmalı.
- Windows gerçek cihazda winget export, uygulama tekrar açılışı, bildirim ve kurulum/kaldırma sınanmalı. macOS Intel paketi ayrıca test edilmeli.
- Apple Developer ile imzalama ve notarization, Windows için uygun kod imzalama sertifikası/servisi yapılandırılmalı. Mevcut yerel paket Developer ID ile imzalı veya noter onaylı değildir.
- İmzalama sırları yalnızca GitHub Secrets veya işletim sisteminin güvenli deposuna girilmeli; kaynak koduna veya sohbet metnine konmamalı. Tauri resmi [macOS imzalama](https://v2.tauri.app/distribute/sign/macos/) ve [Windows imzalama](https://v2.tauri.app/distribute/sign/windows/) belgeleri izlenmeli.
- Depo adı/sahibi ve herkese açık yayın kararı kullanıcıyla kesinleştirilmeli. GitHub Private Vulnerability Reporting etkinleştirilmeli ve SECURITY.md özel raporlama adresi tamamlanmalı.
- Hedefe özgü üçüncü taraf lisans bildirimleri `python scripts/third-party-notices.py --target HEDEF` ile yenilenmeli (aday paket iş akışı bunu yapar). RustSec’in hedef dışı bakım/unsoundness uyarıları her platform için değerlendirilmeli.
- Lisans, kapsam sınırlamaları ve gizlilik açıklaması yayın sayfasında görünür olmalı. “Tüm açıkları bulur” gibi iddialar yapılmamalı.

## Somut sıra

1. Kullanıcının yerel kabul testi ve hataların giderilmesi.
2. Seçilen GitHub deposuna kaynakların yüklenmesi; CI ve aday paket iş akışlarının çalıştırılması.
3. İmzalı paketleri üretme, platform kabul testleri, SHA-256 doğrulaması.
4. Değişiklik notları ve paketlerle taslak GitHub Release hazırlama.
5. Kullanıcıyla taslağı inceleyip genel yayını açma.

Release-candidate iş akışı `contents: read` izniyle yalnızca test artifact’i üretir. Otomatik yayın veya yazılım güncelleme yoktur. Çıktılar kendi başına imzalı üretim dağıtımı sayılmaz.
