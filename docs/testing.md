# 0.2.1 kabul testi

1. `.dmg` veya `.app.zip` paketini açın. İlk açılışta envanter yoksa **Şimdi tara** seçin. Önceden test edilmiş cihazda son tarama geri gelir.
2. Bağımlılıkları göster seçeneğinin başlangıçta kapalı olduğunu, doğrudan kurulan araçların ayrı göründüğünü kontrol edin. Her paketin Tespit kaynağı ve yerel kanıt bölümünü açın. Önceki tarama kaydı güncel durum sayılmamalıdır.
3. Kurulu paketlerden birkaçının sürümünü kendi Homebrew/winget envanterinizle karşılaştırın. Envanter dışındaki uygulamaların listelenmemesi beklenir.
3. Paket aramasını ve inceleme filtresini deneyin. Çevrimdışı modda güvenlik alanı “Kontrol edilmedi” kalmalıdır.
4. **Tarama geçmişi** ekranında yeni kaydı görün. Uygulamayı kapatıp açınca son sonuç ve ayarlar korunmalıdır.
5. **JSON raporu kaydet** ile uygulama veri dizinindeki rapor yolunu alın. Rapor yazılım envanteri içerir; paylaşmadan önce inceleyin.
6. İsterseniz Tercihler’de veri açıklamasını okuyup çevrimiçi kontrolü açın, kaydedin ve yeniden tarayın. Kaynak hatası veya kapsam dışı durumun güvenli sonuç gibi görünmediğini kontrol edin.
7. Periyodik tarama uygulama açıkken yaklaşık 6 saatte bir çalışır. Kısa bir test oturumunda altı saat beklemek gerekmez; zamanlayıcı kuralları birim testleriyle ayrıca sınanır.
8. İsterseniz sistem bildirimlerini açıp işletim sistemi iznini deneyin. Bildirim yalnızca ilgili kayıt veya Node güvenlik sürümü bulunduğunda çıkar. Bildirim bulunmaması temiz tarama anlamına gelmez.

Windows: Microsoft App Installer ve winget kurulu olmalı. İlk kaynak anlaşmaları DevShield tarafından kabul edilmez; winget’i kendiniz bir kez çalıştırıp ilgili anlaşmaları okuyun. Eşlenemeyen uygulamalar export dışında kalabilir. Windows’ta gerçek export, yükleyici ve bildirim testi henüz yapılmadı.

Geri bildirim için işletim sistemi/mimari, uygulama sürümü, yapılan işlem ve görünen hata metni yeterlidir. Tam raporu herkese açık paylaşmayın.

## Geliştirici entegrasyon testi

```sh
# Gerçek yerel envanter + geçici SQLite, ağ sorgusu yok
cargo run --locked --manifest-path src-tauri/Cargo.toml --example smoke
# Aynı yerel test + yalnızca sabit örnek sürümlerle canlı kaynak sorguları
cargo run --locked --manifest-path src-tauri/Cargo.toml --example smoke -- --online
```

Canlı probe gerçek kurulu envanteri ağ fonksiyonlarına geçirmez; Node 22.1.0, curl 8.5.0 ve Git.Git 2.40.0 örneklerini kullanır. Test çıktılarına paket listesi yazılmaz. CI testleri gerçek envanter veya canlı kaynak gerektirmez.
