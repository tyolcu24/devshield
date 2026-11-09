export interface Finding {id:string;summary:string;url:string;source:string;evidence:string}
export interface Package {manager:string;id:string;version:string;install_reason:string;verification:string;evidence_path:string|null;latest:string|null;update_status:string;security_status:string;findings:Finding[]}
export interface Scan {packages:Package[];warnings:string[];checked_at:number;online:boolean}
export interface Settings {online:boolean;periodic:boolean;notifications:boolean}
export interface History {id:number;started_at:number;package_count:number;finding_count:number}
export function statusLabel(count:number):string{return count===0?'Envanter boş; güvenlik sonucu çıkarılamaz.':`${count} paket tespit edildi. Kapsam ve kaynak durumlarını inceleyin.`;}
export const securityLabels:Record<string,string>={not_checked:'Kontrol edilmedi',unverified_inventory:'Yerel kurulum doğrulanmadı',unsupported_identity:'Kimlik / sürüm kapsam dışında',deferred:'Kontrol ertelendi',source_error:'Kaynak hatası',no_known_match:'Kaynakta eşleşme yok',insufficient_evidence:'Sürüm kanıtı yetersiz',review_required:'İnceleme gerekli'};
export const updateLabels:Record<string,string>={unknown:'Bilinmiyor',source_same:'Kaynak sürümüyle aynı',installed_newer:'Kurulu sürüm daha yeni',version_review:'Sürüm karşılaştırması gerekli',catalog_same:'Yerel katalogla aynı',catalog_update_available:'Yerel katalogda yeni sürüm',current_branch:'Node dalında güncel',update_available:'Yeni sürüm bulundu',security_update_available:'Node güvenlik sürümü'};
export function needsAttention(p:Package):boolean{return p.findings.length>0||['update_available','security_update_available','catalog_update_available'].includes(p.update_status);}
export function safeAdvisoryUrl(value:string):string|undefined {try{const u=new URL(value);if(u.protocol==='https:'&&u.hostname==='osv.dev'&&!u.username&&!u.password&&u.pathname.startsWith('/vulnerability/'))return u.href;}catch{}return undefined;}

export const reasonLabels:Record<string,string>={requested:'Doğrudan kurulan araç',dependency:'Başka bir aracın bağımlılığı',unknown:'Kurulum nedeni bilinmiyor'};
export const verificationLabels:Record<string,string>={receipt_found:'Yerel Homebrew kurulum makbuzu bulundu',cask_directory_found:'Yerel Caskroom sürüm dizini bulundu',manager_reported:'Paket yöneticisinin bildirdiği kayıt; dosya doğrulaması yok',missing_receipt:'Yerel kurulum makbuzu bulunamadı',unverified:'Kurulum kanıtı doğrulanamadı',legacy_unverified:'Eski tarama; yeni kanıt kontrolü yapılmamış'};
export function visiblePackage(p:Package,dependencies:boolean,unverified:boolean):boolean{return (dependencies||p.install_reason!=='dependency')&&(unverified||!['missing_receipt','unverified','legacy_unverified'].includes(p.verification));}
