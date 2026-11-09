import {expect,test} from 'vitest';
import {statusLabel,needsAttention,safeAdvisoryUrl,Package,visiblePackage} from './model';
test('empty inventory never implies safety',()=>expect(statusLabel(0)).toContain('çıkarılamaz'));
test('inventory is not proof of safety',()=>expect(statusLabel(2)).toContain('Kapsam'));
test('only source links are rendered',()=>{expect(safeAdvisoryUrl('javascript:alert(1)')).toBeUndefined();expect(safeAdvisoryUrl('https://osv.dev.evil.test/vulnerability/X')).toBeUndefined();expect(safeAdvisoryUrl('https://osv.dev/vulnerability/OSV-1')).toBeTruthy();});
test('security updates need attention',()=>{const p={findings:[],update_status:'security_update_available'} as unknown as Package;expect(needsAttention(p)).toBe(true);expect(needsAttention({...p,update_status:'unknown'})).toBe(false);});

test('dependencies and unverified old snapshots are hidden by default',()=>{const p={install_reason:'dependency',verification:'receipt_found'} as Package;expect(visiblePackage(p,false,false)).toBe(false);expect(visiblePackage(p,true,false)).toBe(true);expect(visiblePackage({...p,verification:'legacy_unverified'},true,false)).toBe(false);expect(visiblePackage({...p,verification:'legacy_unverified'},true,true)).toBe(true);});
