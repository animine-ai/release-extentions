import copy, io, json, sys, unittest, zipfile
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import arex as a

class ToolsTest(unittest.TestCase):
    def setUp(self):
        self.data,self.m=a.build_package(b'\0asm\1\0\0\0','https://github.com/test/fixture','a'*40,a.test_key(4),b'lock')
        self.root,self.index,self.pin=a.fixture_chain(self.data,self.m)
    def resign_index(self,s):return a.envelope(s,[a.test_key(3)],'INDEX')
    def test_positive_chain(self):
        a.verify_root(self.root,self.pin,a.FIXTURE_NOW);a.verify_index(self.index,self.root,self.pin,a.FIXTURE_NOW)
        a.verify_package(self.data,self.index['signed']['entries'][0],self.root,a.FIXTURE_NOW)
    def test_reproducible_package(self):
        d,m=a.build_package(b'\0asm\1\0\0\0','https://github.com/test/fixture','a'*40,a.test_key(4),b'lock');self.assertEqual(d,self.data)
    def test_duplicate_json(self):
        with self.assertRaises(ValueError):a.strict(b'{"a":1,"a":2}')
    def test_invalid_utf8(self):
        with self.assertRaises((ValueError,UnicodeError)):a.strict(b'{"a":"\xff"}')
    def test_surrogate(self):
        with self.assertRaises((ValueError,UnicodeError)):a.strict(b'{"a":"\\ud800"}')
    def test_float_nonfinite(self):
        for value in [b'1.0',b'NaN',b'9007199254740992']:
            with self.assertRaises(ValueError):a.strict(value)
    def test_depth(self):
        with self.assertRaises(ValueError):a.strict(b'['*18+b'0'+b']'*18)
    def test_jcs_utf16_order(self):
        self.assertEqual(a.jcs({'\ue000':1,'😀':2}),'{"😀":2,"\ue000":1}'.encode())
    def test_unknown_manifest_field(self):
        m=copy.deepcopy(self.m);m['unknown']=True
        with self.assertRaises(ValueError):a.validate(m,'Manifest')
    def test_schema_entrypoints(self):
        for p in (a.ROOT/'sdk/schema').glob('*.schema.json'):
            d=json.loads(p.read_text())
            if '$ref' in d:self.assertIn(d['$ref'].split('/')[-1],a.DEFS)
    def test_missing_nullable(self):
        r={'requestId':'r','sourceRole':'CALENDAR','url':'u','method':'GET'}
        with self.assertRaises(ValueError):a.validate(r,'RequestSpec')
    def test_utf8_byte_bound(self):
        with self.assertRaises(ValueError):a.validate({'code':'x','message':'😀'*100},'ObservationDiagnosticV1')
    def test_wrong_root_pin(self):
        pin=dict(self.pin,initialRootSha256='0'*64)
        with self.assertRaises(ValueError):a.verify_root(self.root,pin,a.FIXTURE_NOW)
    def test_root_threshold(self):
        r=copy.deepcopy(self.root);r['signatures']=r['signatures'][:1]
        with self.assertRaises(ValueError):a.verify_root(r,self.pin,a.FIXTURE_NOW)
    def test_duplicate_signer(self):
        r=copy.deepcopy(self.root);r['signatures']=[r['signatures'][0]]*2
        with self.assertRaises(ValueError):a.verify_root(r,self.pin,a.FIXTURE_NOW)
    def test_index_expiry_and_future(self):
        for updates in [{'expiresAt':'2026-09-28T00:00:00Z'},{'issuedAt':'2026-09-30T12:00:00Z'},{'expiresAt':'2030-01-01T00:00:00Z'}]:
            s=copy.deepcopy(self.index['signed']);s.update(updates)
            with self.assertRaises(ValueError):a.verify_index(self.resign_index(s),self.root,self.pin,a.FIXTURE_NOW)
    def test_replay_same_digest_allowed(self):a.verify_index(self.index,self.root,self.pin,a.FIXTURE_NOW,self.index)
    def test_index_equivocation(self):
        s=copy.deepcopy(self.index['signed']);s['entries'][0]['displayName']='Changed'
        with self.assertRaises(ValueError):a.verify_index(self.resign_index(s),self.root,self.pin,a.FIXTURE_NOW,self.index)
    def test_index_downgrade(self):
        s=copy.deepcopy(self.index['signed']);s['sequence']=2
        higher=self.resign_index(s)
        with self.assertRaises(ValueError):a.verify_index(self.index,self.root,self.pin,a.FIXTURE_NOW,higher)
    def test_origin_and_immutable_url(self):
        for url in ['http://packages.example.org/x','https://other.example.org/'+a.sha(self.data)+'.arex','https://packages.example.org/dist/current.arex','https://packages.example.org/dist/%2e%2e/'+a.sha(self.data)+'.arex']:
            s=copy.deepcopy(self.index['signed']);s['entries'][0]['packageUrl']=url
            with self.assertRaises(ValueError):a.verify_index(self.resign_index(s),self.root,self.pin,a.FIXTURE_NOW)
    def test_scope(self):
        s=copy.deepcopy(self.index['signed']);s['entries'][0]['providerId']='outside'
        with self.assertRaises(ValueError):a.verify_index(self.resign_index(s),self.root,self.pin,a.FIXTURE_NOW)
    def test_yank_and_revocation(self):
        for field in ['yanked','revoked']:
            e=copy.deepcopy(self.index['signed']['entries'][0]);e[field]=True
            with self.assertRaises(ValueError):a.verify_package(self.data,e,self.root,a.FIXTURE_NOW)
    def test_tampered_archive(self):
        bad=self.data[:-1]+bytes([self.data[-1]^1])
        with self.assertRaises(ValueError):a.verify_package(bad,self.index['signed']['entries'][0],self.root,a.FIXTURE_NOW)
    def test_root_only_and_symlink(self):
        for name,mode in [('../module.wasm',0o100644),('module.wasm',0o120777)]:
            b=io.BytesIO();files=a.read_archive(self.data)
            with zipfile.ZipFile(b,'w') as z:
                for n,d in files.items():
                    e=zipfile.ZipInfo(name if n=='module.wasm' else n);e.create_system=3;e.external_attr=mode<<16;z.writestr(e,d)
            with self.assertRaises(ValueError):a.read_archive(b.getvalue())
    def test_zip_prefix_trailer_and_local_header_binding(self):
        for bad in [b'prefix'+self.data,self.data+b'trailer',self.data[:26]+b'\0\0'+self.data[28:]]:
            with self.assertRaises((ValueError,zipfile.BadZipFile)):a.read_archive(bad)
    def test_full_64_unicode_display_name(self):
        m=copy.deepcopy(self.m);m['displayName']='😀'*64;a.validate(m,'Manifest')
        m['displayName']='😀'*65
        with self.assertRaises(ValueError):a.validate(m,'Manifest')
    def test_rotation_dual_threshold(self):
        r=rotation(self.root)
        a.verify_root(r,self.pin,a.FIXTURE_NOW,self.root)
        for sigs in [r['signatures'][:2],r['signatures'][2:]]:
            bad=copy.deepcopy(r);bad['signatures']=sigs
            with self.assertRaises(ValueError):a.verify_root(bad,self.pin,a.FIXTURE_NOW,self.root)
    def test_root_version_gap(self):
        s=rotation(self.root)['signed'];s['version']=3
        r=a.envelope(s,[a.test_key(i) for i in [0,1,5,6]],'ROOT')
        with self.assertRaises(ValueError):a.verify_root(r,self.pin,a.FIXTURE_NOW,self.root)
    def test_revocation_rollback(self):
        prev=copy.deepcopy(self.root);prev['signed']['revokedDigests']=['f'*64]
        with self.assertRaises(ValueError):a.verify_root(rotation(self.root),self.pin,a.FIXTURE_NOW,prev)

def rotation(root):
    s=copy.deepcopy(root['signed']);s['version']=2
    keys=[a.test_key(i) for i in [3,4,5,6,7]]
    s['keys']=[{'keyId':a.key_id(k),'publicKey':a.b64(a.public(k))} for k in keys]
    s['roles']['root']['keyIds']=[a.key_id(a.test_key(i)) for i in [5,6,7]]
    return a.envelope(s,[a.test_key(i) for i in [0,1,5,6]],'ROOT')
if __name__=='__main__':unittest.main()
