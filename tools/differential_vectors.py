"""Differential vectors for publisher tool (arex.py) versus the original host verifiers.

Usage: python3 tools/differential_vectors.py OUTPUT_DIR FIXTURE_WASM

Writes hostile strict-JSON documents, display names, ZIP mutations and publisher-scope cases together with the
verdict of this repository's verifier. compatibility/host/.../DifferentialParityTest.kt feeds the same bytes to the
unmodified host code and asserts that both sides agree, except for the explicitly listed benign differences.
No network, no production keys: every key is a public test seed.
"""
import base64, copy, io, json, shutil, stat, struct, sys, zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import arex  # noqa: E402

SOURCE_REPOSITORY = 'https://github.com/animine-ai/release-extentions'
SOURCE_COMMIT = '0' * 40


def b64(data):
    return base64.b64encode(data).decode()


def nested(levels):
    return '{"a":' + '[' * (levels - 1) + ']' * (levels - 1) + '}'


def json_vectors():
    vectors = []

    def add(name, data):
        vectors.append((name, data.encode('utf-8', 'surrogatepass') if isinstance(data, str) else data))

    add('simple', '{"a":1}')
    add('empty-obj', '{}')
    add('ws-padded', ' \t\r\n{"a":1}\n ')
    add('dup', '{"a":1,"a":2}')
    add('dup-escaped', '{"a":1,"\\u0061":2}')
    add('dup-escaped-nonascii', '{"\\u00e9":1,"\u00e9":2}')
    add('bom', b'\xef\xbb\xbf{"a":1}')
    add('float-1.0', '{"a":1.0}')
    add('float-exp', '{"a":1e2}')
    add('neg-zero', '{"a":-0}')
    add('float-half', '{"a":0.5}')
    add('max-safe', '{"a":9007199254740991}')
    add('over-safe', '{"a":9007199254740992}')
    add('neg-max-safe', '{"a":-9007199254740991}')
    add('huge-int', '{"a":123456789012345678901234567890}')
    add('nan', '{"a":NaN}')
    add('inf', '{"a":Infinity}')
    add('ninf', '{"a":-Infinity}')
    add('lone-hi-esc', '{"a":"\\ud800"}')
    add('lone-lo-esc', '{"a":"\\udc00"}')
    add('pair-esc', '{"a":"\\ud83d\\ude00"}')
    add('pair-raw', '{"a":"\U0001F600"}')
    add('bad-utf8-ff', b'{"a":"\xff"}')
    add('overlong-utf8', b'{"a":"\xc0\x80"}')
    add('truncated-utf8', b'{"a":"\xe3\x81"}')
    add('cesu-surrogate-utf8', b'{"a":"\xed\xa0\x80"}')
    add('nul-esc', '{"a":"\\u0000"}')
    add('raw-tab-in-string', '{"a":"\t"}')
    add('raw-lf-in-string', '{"a":"x\ny"}')
    add('esc-1f', '{"a":"\\u001f"}')
    add('del-raw', '{"a":"\x7f"}')
    add('u2028-raw', '{"a":"\u2028"}')
    for depth in (16, 17, 18, 19):
        add('depth-%d' % depth, nested(depth))
    add('trail-comma', '{"a":1,}')
    add('trail-garbage', '{"a":1} x')
    add('comment', '{"a":1/*x*/}')
    add('single-quote', "{'a':1}")
    add('unquoted-key', '{a:1}')
    add('leading-zero', '{"a":01}')
    add('plus-number', '{"a":+1}')
    add('hex-number', '{"a":0x10}')
    add('dot-trailing', '{"a":1.}')
    add('dot-leading', '{"a":.5}')
    add('nbsp-ws', '\u00a0{"a":1}')
    add('vt-ws', '\x0b{"a":1}')
    add('ff-ws', '\x0c{"a":1}')
    add('key-order-bmp-vs-supplementary', '{"\uffff":1,"\U00010000":2,"a":3,"B":4,"\ue000":5}')
    add('key-order-escapes', '{"\\u00e9":1,"e":2,"Z":3,"z":4,"\\u0080":5}')
    add('rfc8785-section-3.2.3-keys',
        '{"\u20ac":"Euro Sign","\r":"Carriage Return","\ufb33":"Hebrew Letter Dalet With Dagesh","1":"One",'
        '"\U0001F600":"Emoji: Grinning Face","\u0080":"Control","\u00f6":"Latin Small Letter O With Diaeresis"}')
    add('slash-escape', '{"a":"\\/"}')
    add('all-short-escapes', '{"a":"\\b\\f\\n\\r\\t\\"\\\\"}')
    add('empty-key', '{"":1}')
    add('nfc-vs-nfd-keys', '{"\u00e9":1,"e\u0301":2}')
    add('nested-ok', '{"a":{"b":[1,2,{"c":null}],"d":true}}')
    add('top-array', '[1]')
    add('top-number', '1')
    add('top-string', '"x"')
    add('top-null', 'null')
    add('empty-input', '')
    add('array-4096', '{"a":[' + ','.join(['1'] * 4096) + ']}')
    add('array-4097', '{"a":[' + ','.join(['1'] * 4097) + ']}')
    add('obj-256-fields', '{' + ','.join('"k%d":1' % i for i in range(256)) + '}')
    add('obj-257-fields', '{' + ','.join('"k%d":1' % i for i in range(257)) + '}')
    add('noncharacters', '{"a":"\ufffe\uffff"}')
    add('combining-and-zwj', '{"a":"e\u0301 \u200d \u200b"}')
    add('big-string-70k', '{"a":"' + 'x' * 70000 + '"}')
    head = '{"a":"'
    pad = 262144 - len(head) - 2
    add('exact-cap-262144', head + 'x' * pad + '"}')
    add('cap-plus-1', head + 'x' * (pad + 1) + '"}')
    return vectors


DISPLAY_NAMES = [
    'AniWorld', 'Fixture Provider', '\u00dcn\u00efc\u00f6d\u00e9', '\u65e5\u672c\u8a9e\u30a2\u30cb\u30e1',
    '\U0001F600 emoji', 'e\u0301 combining', 'a\u2028b line separator', 'a\u2029b paragraph separator',
    '\u200bzero width space', '\u200eLRM', '\u200fRLM', '\u061cALM', '\ufeffBOM', '\ufffe\uffff noncharacters',
    '<script>alert(1)</script>', '  padded  ', '\u00a0nbsp', 'a\u0085b NEL', 'a\tb tab', 'a\nb newline',
    'a\x7fb del', 'a\x00b nul', '\u202eRLO', '\u202aLRE', '\u2066LRI', '\u2069PDI',
    'x' * 64, 'x' * 65, '\U0001F600' * 64, '\U0001F600' * 65, '\u3042' * 64, '\u3042' * 65,
    'a"b\\c', 'a/b', '\ud7ff\ue000', '\U0010ffff', '\U00010000', 'zalgo\u0300\u0301\u0302\u0303',
]


def main(out_dir, fixture_wasm):
    out = Path(out_dir)
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir(parents=True)
    module = Path(fixture_wasm).read_bytes()
    key = arex.test_key(4)
    lock = (ROOT / 'Cargo.lock').read_bytes()
    base_data, base_manifest = arex.build_package(module, SOURCE_REPOSITORY, SOURCE_COMMIT, key, lock)
    files = arex.read_archive(base_data)
    provenance = json.loads(files['provenance.json'])
    notice = files['NOTICE']

    # ------------------------------------------------------------------ A. strict JSON + canonical bytes
    records = []
    for name, data in json_vectors():
        try:
            value = arex.strict(data, 262144)
            accepted = isinstance(value, dict)
            canonical = arex.jcs(value).hex() if accepted else None
            error = None if accepted else 'top-level is not an object'
        except Exception as exc:  # noqa: BLE001 - every rejection reason is data
            accepted, canonical, error = False, None, str(exc)
        records.append({'name': name, 'b64': b64(data), 'pyAccept': accepted, 'pyCanon': canonical, 'pyErr': error})
    (out / 'json-vectors.json').write_text(json.dumps(records))

    def write_case(directory, data, manifest, with_archive=True):
        directory.mkdir(parents=True)
        root, index, pin = arex.fixture_chain(data, manifest)
        if with_archive:
            (directory / 'fixture.arex').write_bytes(data)
        for name, value in [('root.json', root), ('index.json', index), ('test-pin.json', pin)]:
            (directory / name).write_bytes(arex.jcs(value))
        return root, index, pin

    def python_verdict(data, root, index, pin):
        try:
            arex.verify_root(root, pin, arex.FIXTURE_NOW)
            arex.verify_index(index, root, pin, arex.FIXTURE_NOW)
            arex.verify_package(data, index['signed']['entries'][0], root, arex.FIXTURE_NOW)
            return True, None
        except Exception as exc:  # noqa: BLE001
            return False, str(exc)

    # ------------------------------------------------------------------ B. display names
    names = []
    for i, name in enumerate(DISPLAY_NAMES):
        entry = {'dir': 'names/%03d' % i, 'name': name, 'pyAccept': False, 'pyErr': None}
        try:
            manifest = copy.deepcopy(base_manifest)
            manifest['displayName'] = name
            data, signed_manifest = arex.assemble_package(module, manifest, provenance, notice, key)
            root, index, pin = write_case(out / entry['dir'], data, signed_manifest)
            entry['pyAccept'], entry['pyErr'] = python_verdict(data, root, index, pin)
        except Exception as exc:  # noqa: BLE001
            entry['pyErr'] = 'build: ' + str(exc)
        names.append(entry)
    (out / 'names.json').write_text(json.dumps(names))

    # ------------------------------------------------------------------ C. ZIP mutations
    members = ['manifest.json', 'module.wasm', 'provenance.json', 'NOTICE', 'package.sig']
    original = {n: files[n] for n in members}

    def zip_bytes(entries, compression=zipfile.ZIP_STORED, comment=b'', mutate=None, unseekable=False, zip64=False):
        buffer = io.BytesIO()
        target = buffer
        if unseekable:
            class NoSeek(io.RawIOBase):
                def __init__(self):
                    self.buf = io.BytesIO()

                def writable(self):
                    return True

                def write(self, b):
                    return self.buf.write(b)

                def tell(self):
                    return self.buf.tell()
            target = NoSeek()
        with zipfile.ZipFile(target, 'w', compression=compression, allowZip64=zip64) as z:
            z.comment = comment
            for name, data in entries:
                info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
                info.create_system = 3
                info.external_attr = 0o100644 << 16
                info.compress_type = compression
                if mutate:
                    mutate(name, info)
                z.writestr(info, data)
        return target.buf.getvalue() if unseekable else buffer.getvalue()

    def entries(skip=(), extra=(), rename=None):
        return [((rename or {}).get(n, n), original[n]) for n in members if n not in skip] + list(extra)

    def patched(data, fn):
        b = bytearray(data)
        fn(b)
        return bytes(b)

    def central_positions(data):
        end = struct.unpack_from('<4s4H2IH', data, len(data) - 22)
        offset, size = end[6], end[5]
        pos, found = offset, []
        while pos < offset + size:
            name_len, extra_len, comment_len = struct.unpack_from('<3H', data, pos + 28)
            found.append(pos)
            pos += 46 + name_len + extra_len + comment_len
        return found

    def set_flag(bit):
        def apply(b):
            for pos in central_positions(bytes(b)):
                struct.pack_into('<H', b, pos + 8, struct.unpack_from('<H', b, pos + 8)[0] | bit)
        return apply

    base = zip_bytes(entries())
    cases = {
        'baseline-stored': base,
        'all-deflated': zip_bytes(entries(), compression=zipfile.ZIP_DEFLATED),
        'unseekable-data-descriptors': zip_bytes(entries(), unseekable=True),
        'extra-entry': zip_bytes(entries(extra=[('extra.txt', b'x')])),
        'missing-notice': zip_bytes(entries(skip=('NOTICE',))),
        'directory-entry': zip_bytes(entries(extra=[('dir/', b'')])),
        'case-variant-name': zip_bytes(entries(rename={'NOTICE': 'notice'})),
        'traversal-name': zip_bytes(entries(rename={'manifest.json': '../manifest.json'})),
        'backslash-name': zip_bytes(entries(rename={'NOTICE': 'sub\\NOTICE'})),
        'leading-slash-name': zip_bytes(entries(rename={'NOTICE': '/NOTICE'})),
        'dup-entry': zip_bytes(entries(extra=[('NOTICE', b'second notice')])),
        'archive-comment': zip_bytes(entries(), comment=b'hi'),
        'extra-field': zip_bytes(entries(), mutate=lambda n, i: setattr(i, 'extra', b'\x55\x54\x05\x00\x01\x00\x00\x00\x00') if n == 'NOTICE' else None),
        'entry-comment': zip_bytes(entries(), mutate=lambda n, i: setattr(i, 'comment', b'c') if n == 'NOTICE' else None),
        'symlink-mode': zip_bytes(entries(), mutate=lambda n, i: setattr(i, 'external_attr', (stat.S_IFLNK | 0o777) << 16) if n == 'NOTICE' else None),
        'create-system-windows': zip_bytes(entries(), mutate=lambda n, i: (setattr(i, 'create_system', 0), setattr(i, 'external_attr', 0))),
        'exec-permission': zip_bytes(entries(), mutate=lambda n, i: setattr(i, 'external_attr', 0o100755 << 16)),
        'encrypted-flag': patched(base, set_flag(1)),
        'utf8-flag-0x800': patched(base, set_flag(0x800)),
        'crc-corrupt-central': patched(base, lambda b: b.__setitem__(central_positions(bytes(b))[3] + 16, b[central_positions(bytes(b))[3] + 16] ^ 1)),
        'central-size-lie': patched(base, lambda b: struct.pack_into('<I', b, central_positions(bytes(b))[0] + 24, struct.unpack_from('<I', b, central_positions(bytes(b))[0] + 24)[0] + 1)),
        'overlapping-offsets': patched(base, lambda b: struct.pack_into('<I', b, central_positions(bytes(b))[4] + 42, struct.unpack_from('<I', b, central_positions(bytes(b))[3] + 42)[0])),
        'trailing-garbage': base + b'GARBAGE',
        'leading-garbage': b'JUNK' + base,
        'reordered-entries': zip_bytes(list(reversed(entries()))),
        'empty-notice': zip_bytes([(n, b'' if n == 'NOTICE' else d) for n, d in entries()]),
        'module-zero-bomb-20MiB': zip_bytes(entries(skip=('module.wasm',), extra=[('module.wasm', b'\x00' * (20 * 1024 * 1024))]), compression=zipfile.ZIP_DEFLATED),
        'stored-wrong-method-bits': patched(base, lambda b: [struct.pack_into('<H', b, p + 10, 99) for p in central_positions(bytes(b))]),
    }
    archives = []
    for i, (name, data) in enumerate(cases.items()):
        entry = {'dir': 'archives/%03d' % i, 'name': name, 'pyAccept': False, 'pyErr': None}
        try:
            root, index, pin = write_case(out / entry['dir'], data, base_manifest)
            entry['pyAccept'], entry['pyErr'] = python_verdict(data, root, index, pin)
        except Exception as exc:  # noqa: BLE001
            entry['pyErr'] = 'build: ' + str(exc)
        archives.append(entry)
    (out / 'archives.json').write_text(json.dumps(archives))

    # ------------------------------------------------------------------ D. publisher-scope windows (whole-catalog effects)
    keys = [arex.test_key(i) for i in range(5)]
    base_root = arex.fixture_chain(base_data, base_manifest)[0]
    base_index = arex.fixture_chain(base_data, base_manifest)[1]
    scopes = []
    for name, mutation in [('second-scope-valid', {}),
                           ('second-scope-expired', {'expiresAt': '2026-09-01T00:00:00Z'}),
                           ('second-scope-not-yet-valid', {'notBefore': '2026-09-29T12:05:00Z'})]:
        signed_root = copy.deepcopy(base_root['signed'])
        other = copy.deepcopy(signed_root['publishers'][0])
        other.update({'extensionId': 'old.ext', 'providerId': 'oldprov'})
        other.update(mutation)
        signed_root['publishers'].append(other)
        root_env = arex.envelope(signed_root, keys[:2], 'ROOT')
        pin = dict(arex.fixture_chain(base_data, base_manifest)[2], initialRootSha256=arex.sha(arex.jcs(signed_root)))
        signed_index = copy.deepcopy(base_index['signed'])
        second = copy.deepcopy(signed_index['entries'][0])
        second.update({'extensionId': 'old.ext', 'providerId': 'oldprov', 'releaseSequence': 1})
        signed_index['entries'].append(second)
        index_env = arex.envelope(signed_index, [keys[3]], 'INDEX')
        directory = out / 'scopes' / name
        directory.mkdir(parents=True)
        for file_name, value in [('root.json', root_env), ('index.json', index_env), ('test-pin.json', pin)]:
            (directory / file_name).write_bytes(arex.jcs(value))
        try:
            arex.verify_root(root_env, pin, arex.FIXTURE_NOW)
            arex.verify_index(index_env, root_env, pin, arex.FIXTURE_NOW)
            accepted, error = True, None
        except Exception as exc:  # noqa: BLE001
            accepted, error = False, str(exc)
        scopes.append({'dir': 'scopes/' + name, 'name': name, 'pyAccept': accepted, 'pyErr': error})
    (out / 'scopes.json').write_text(json.dumps(scopes))
    print('json=%d names=%d archives=%d scopes=%d' % (len(records), len(names), len(archives), len(scopes)))


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    main(sys.argv[1], sys.argv[2])
