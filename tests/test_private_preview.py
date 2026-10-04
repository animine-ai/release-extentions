import sys, tempfile, unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import arex as a
import private_preview as p

SEED = b'a-test-seed-that-is-long-enough-for-the-derivation-0123456789'
MODULE = b'\0asm\1\0\0\0'
NOW = datetime(2026, 10, 4, 12, 0, tzinfo=timezone.utc)
ORIGIN = 'https://raw.githubusercontent.com'
BASE = '/example/catalog-branch/catalog'

class PrivatePreviewTest(unittest.TestCase):
    def chain(self, seed=SEED, commit='d' * 40, sequence=7, index_sequence=1000, now=NOW):
        return p.build_chain(seed, MODULE, 'https://github.com/example/repo', commit, sequence, '1.0.0-preview.%d' % sequence, ORIGIN, BASE, now, index_sequence)

    def test_chain_verifies_with_the_unchanged_strict_rules(self):
        data, root, index, pin = self.chain()
        a.verify_root(root, pin, NOW)
        a.verify_index(index, root, pin, NOW)
        manifest = a.verify_package(data, index['signed']['entries'][0], root, NOW)
        self.assertEqual((manifest['extensionId'], manifest['providerId'], manifest['allowedHosts']), ('de.aniworld', 'aniworld', ['aniworld.to']))
        self.assertEqual(root['signed']['roles']['root']['threshold'], 2)
        self.assertEqual(len(root['signed']['roles']['root']['keyIds']), 3)
        url = index['signed']['entries'][0]['packageUrl']
        self.assertTrue(url.startswith(ORIGIN + BASE + '/dist/de.aniworld/1.0.0-preview.7/') and url.endswith(a.sha(data) + '.arex'))

    def test_the_root_and_its_fingerprint_never_change_between_publications(self):
        _, root_a, _, _ = self.chain()
        _, root_b, _, _ = self.chain(commit='e' * 40, sequence=8, index_sequence=2000, now=NOW + timedelta(days=3))
        self.assertEqual(a.jcs(root_a), a.jcs(root_b))
        self.assertEqual(p.fingerprint(root_a), p.fingerprint(root_b))

    def test_a_package_is_reproducible_and_the_index_is_re_signed(self):
        data_a, _, index_a, _ = self.chain()
        data_b, _, index_b, _ = self.chain(index_sequence=1001, now=NOW + timedelta(days=1))
        self.assertEqual(data_a, data_b)
        self.assertLess(index_a['signed']['sequence'], index_b['signed']['sequence'])
        self.assertLessEqual(utc(index_b['signed']['expiresAt']) - utc(index_b['signed']['issuedAt']), timedelta(days=7))

    def test_a_different_seed_is_a_different_identity_and_a_short_seed_is_refused(self):
        _, root_a, _, _ = self.chain()
        _, root_b, _, _ = self.chain(seed=b'another-seed-that-is-also-long-enough-0123456789abcdef')
        self.assertNotEqual(p.fingerprint(root_a), p.fingerprint(root_b))
        self.assertTrue(set(k['keyId'] for k in root_a['signed']['keys']).isdisjoint(k['keyId'] for k in root_b['signed']['keys']))
        with self.assertRaises(ValueError):
            p.derive_keys(b'short')

    def test_no_private_key_material_and_no_test_seed_reaches_the_output(self):
        data, root, index, _ = self.chain()
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            p.write_catalog(out, data, root, index, '1.0.0-preview.7')
            written = b''.join(f.read_bytes() for f in out.rglob('*') if f.is_file())
        for label, key in p.derive_keys(SEED).items():
            private = key.private_bytes_raw()
            self.assertNotIn(private, written, label)
            self.assertNotIn(a.b64(private).encode(), written, label)
        for i in range(15):  # the public EP03 test seeds are not used
            self.assertNotIn(a.key_id(a.test_key(i)).encode(), written)

    def test_an_expired_index_and_a_tampered_package_are_rejected(self):
        data, root, index, pin = self.chain()
        with self.assertRaises(Exception):
            a.verify_index(index, root, pin, NOW + timedelta(days=7))
        tampered = bytearray(data); tampered[-1] ^= 1
        with self.assertRaises(Exception):
            a.verify_package(bytes(tampered), index['signed']['entries'][0], root, NOW)

def utc(text):
    return datetime.fromisoformat(text.replace('Z', '+00:00'))

if __name__ == '__main__':
    unittest.main()
