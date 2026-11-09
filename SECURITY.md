# Güvenlik politikası

0.2.x şu an test adayıdır; güvenlik garantisi vermez. Güvenlik verisi kapsamı README’de belirtilmiştir. Bilinmeyen/eşleşmeyen sonuçlar güvenli sonuç olarak sunulmaz. Paket yükseltmesi ve keyfi komut çalıştırma yoktur.

Güvenlik açıklarını [GitHub özel güvenlik bildirimi](https://github.com/tyolcu24/devshield/security/advisories/new) üzerinden depo yöneticilerine özel bildirin. Bu özellik etkinleştirilmiştir. Bir açık bulursanız exploit ayrıntılarını, gizli anahtarları veya tam envanterinizi herkese açık issue’ya koymayın. Önce depo sahibinden özel kanal isteyin.

Güven sınırları ve bilinen sınırlamalar: [mimari](docs/architecture.md). Bağımlılık denetimleri CI’da npm audit ve cargo audit ile çalıştırılır.
