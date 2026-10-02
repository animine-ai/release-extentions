import copy, io, json, sys, tempfile, unittest, zipfile
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
    def test_aniworld_test_package_identity_and_reproducibility(self):
        module=b'\0asm\1\0\0\0';source='https://github.com/test/aniworld';commit='b'*40;key=a.test_key(14);lock=(a.ROOT/'Cargo.lock').read_bytes()
        data,m=a.build_aniworld_test_package(module,source,commit,key,lock)
        repeated,repeated_manifest=a.build_aniworld_test_package(module,source,commit,key,lock)
        self.assertEqual((repeated,repeated_manifest),(data,m))
        root,index,pin=a.aniworld_test_chain(data,m);a.verify_root(root,pin,a.FIXTURE_NOW);a.verify_index(index,root,pin,a.FIXTURE_NOW)
        verified=a.verify_aniworld_test_chain(data,index,root,pin,a.FIXTURE_NOW)
        self.assertEqual((verified['extensionId'],verified['providerId'],verified['displayName']),('de.aniworld','aniworld','AniWorld'))
        self.assertEqual(set(verified['capabilities']),set(a.ANIWORLD_ROLES))
        self.assertEqual(set(verified['navigationCapabilities']),set(a.ANIWORLD_NAVIGATION))
        self.assertEqual(verified['allowedHosts'],['aniworld.to'])
    def test_aniworld_test_chain_is_reproducible(self):
        with tempfile.TemporaryDirectory() as first,tempfile.TemporaryDirectory() as second:
            args=(b'\0asm\1\0\0\0','https://github.com/test/aniworld','c'*40)
            a.write_aniworld_test_chain(Path(first),*args);a.write_aniworld_test_chain(Path(second),*args)
            left={p.name:p.read_bytes() for p in Path(first).iterdir()};right={p.name:p.read_bytes() for p in Path(second).iterdir()}
            self.assertEqual(left,right)
            self.assertIn('aniworld-test.arex',left)
    def test_aniworld_test_verifier_rejects_extra_host(self):
        module=b'\0asm\1\0\0\0';source='https://github.com/test/aniworld';commit='d'*40;key=a.test_key(14);lock=(a.ROOT/'Cargo.lock').read_bytes()
        data,m=a.build_aniworld_test_package(module,source,commit,key,lock);files=a.read_archive(data)
        m=copy.deepcopy(m);m['allowedHosts'].append('example.org')
        bad,bad_manifest=a.assemble_package(module,m,a.strict(files['provenance.json']),files['NOTICE'],key)
        root,index,pin=a.aniworld_test_chain(bad,bad_manifest);a.verify_root(root,pin,a.FIXTURE_NOW);a.verify_index(index,root,pin,a.FIXTURE_NOW)
        with self.assertRaisesRegex(ValueError,'AniWorld exact host grant'):
            a.verify_aniworld_test_chain(bad,index,root,pin,a.FIXTURE_NOW)
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
class RepositoryLintTest(unittest.TestCase):
    URL='https://packages.example.org/extensions'
    def setUp(self):
        data,m=a.build_package(b'\0asm\1\0\0\0','https://github.com/test/fixture','a'*40,a.test_key(4),b'lock')
        self.root,self.index,self.pin=a.fixture_chain(data,m)
    def lint(self,url=None,root=None,index=None,pin=None):
        return a.lint_repository(url or self.URL,a.jcs(root or self.root),a.jcs(index or self.index),pin or self.pin,a.FIXTURE_NOW)
    def test_clean_repository_has_no_findings(self):
        self.assertEqual(self.lint(),([],[]))
    def test_url_rules_match_app_registry(self):
        origins={'https://packages.example.org'}
        self.assertEqual(a.normalize_repository_url(' https://Packages.Example.org:443/a/b/ ',origins),'https://packages.example.org/a/b')
        for bad in ['http://packages.example.org/x','https://user@packages.example.org/x','https://packages.example.org/x?y=1',
                    'https://packages.example.org/x#f','https://packages.example.org:8443/x','https://packages.example.org/a//b',
                    'https://packages.example.org/a/../b','https://packages.example.org/%41','https://packages.example.org\\x',
                    'https://127.0.0.1/x','https://localhost/x','https://other.example.org/x','https://packages.example.org/'+'a'*2100]:
            with self.assertRaises(ValueError,msg=bad):a.normalize_repository_url(bad,origins)
    def test_oversized_documents_are_errors(self):
        root=copy.deepcopy(self.root);root['signed']['padding']='x'*70000
        errors,_=self.lint(root=root)
        self.assertTrue(any('root.json' in e and 'fetches at most' in e for e in errors))
        index=copy.deepcopy(self.index);index['signed']['padding']='x'*270000
        errors,_=self.lint(index=index)
        self.assertTrue(any('index.json' in e and 'fetches at most' in e for e in errors))
    def test_origin_must_be_pinned(self):
        errors,_=self.lint(url='https://elsewhere.example.org/extensions')
        self.assertTrue(any('independently authenticated' in e for e in errors))
    def test_scope_expiring_before_index_warns_about_whole_catalog(self):
        signed=copy.deepcopy(self.root['signed']);signed['publishers'][0]['expiresAt']='2026-10-01T00:00:00Z'
        root=a.envelope(signed,[a.test_key(0),a.test_key(1)],'ROOT');pin=dict(self.pin,initialRootSha256=a.sha(a.jcs(signed)))
        errors,warnings=self.lint(root=root,pin=pin)
        self.assertEqual(errors,[]);self.assertTrue(any('WHOLE index' in w for w in warnings),warnings)
    def test_index_close_to_expiry_warns(self):
        signed=copy.deepcopy(self.index['signed']);signed['issuedAt']='2026-09-27T12:00:00Z';signed['expiresAt']='2026-09-30T00:00:00Z'
        index=a.envelope(signed,[a.test_key(3)],'INDEX')
        errors,warnings=self.lint(index=index)
        self.assertEqual(errors,[]);self.assertTrue(any('index expires in' in w for w in warnings),warnings)
    def test_expired_index_is_an_error(self):
        signed=copy.deepcopy(self.index['signed']);signed['issuedAt']='2026-09-20T12:00:00Z';signed['expiresAt']='2026-09-27T12:00:00Z'
        index=a.envelope(signed,[a.test_key(3)],'INDEX')
        errors,_=self.lint(index=index)
        self.assertTrue(any(e.startswith('index:') for e in errors),errors)
    def test_array_entry_limit_matches_host_codec(self):
        with self.assertRaises(ValueError):a.strict(b'{"a":['+b','.join([b'1']*4097)+b']}')
        a.strict(b'{"a":['+b','.join([b'1']*4096)+b']}')
if __name__=='__main__':unittest.main()
