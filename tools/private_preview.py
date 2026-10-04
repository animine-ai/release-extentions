#!/usr/bin/env python3
"""PRIVATE PREVIEW publication (temporary, operator-held keys, manually trusted by the user, NOT independently verified).

This is not the production release path (release.yml stays blocked). It builds a normal AREX v1 chain (2-of-3 root, index key,
delegated publisher key) so the app's unchanged strict verifier can check it, but the keys are derived from one secret seed held
by the operator, not from independent custodians. A valid signature therefore only means "matches the identity the user accepted".

Private keys never leave this process: they are derived in memory from the seed, never written to a file, argument or log.
Only public material (root.json, index.json, the .arex package) is written.
"""
from __future__ import annotations
import argparse, hashlib, hmac, json, os, sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import arex as a
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

REPOSITORY_ID = 'private.preview.animetracker'
PUBLISHER_ID = 'private.preview.publisher'
WORKFLOW = 'private-preview/v1'
ROOT_EXPIRES = '2028-01-01T00:00:00Z'      # fixed, so root.json (and its fingerprint) never changes between publications
PUBLISHER_NOT_BEFORE = '2026-01-01T00:00:00Z'
INDEX_VALIDITY = timedelta(days=6)         # the verifier caps an index at seven days
LABELS = ('root-1', 'root-2', 'root-3', 'index-1', 'publisher-1')
NOTICE_HEAD = (b'AREX PRIVATE PREVIEW: de.aniworld package signed with operator-held keys. It is only usable after the user '
               b'explicitly accepted this source. It is not independently verified and not a production release.\n\n')

def derive_keys(seed: bytes) -> dict[str, Ed25519PrivateKey]:
    """Stable, secret keys from one seed. The seed is the only custody item; losing it means a new identity."""
    if len(seed) < 32:
        raise ValueError('PREVIEW_KEY_SEED must have at least 32 bytes of entropy')
    return {label: Ed25519PrivateKey.from_private_bytes(hmac.new(seed, b'AREX-PRIVATE-PREVIEW-V1:' + label.encode(), hashlib.sha256).digest())
            for label in LABELS}

def utc(text: str) -> datetime:
    return datetime.fromisoformat(text.replace('Z', '+00:00'))

def iso(moment: datetime) -> str:
    return moment.astimezone(timezone.utc).replace(microsecond=0).isoformat().replace('+00:00', 'Z')

def build_package(module: bytes, source_repo: str, source_commit: str, release_sequence: int, version: str, key, lock: bytes):
    a.check(module.startswith(b'\x00asm\x01\x00\x00\x00'), 'core wasm required')
    notice_path = a.ROOT / 'sources/aniworld/NOTICE'
    a.check(notice_path.is_file(), 'AniWorld NOTICE missing')
    notice = NOTICE_HEAD + notice_path.read_bytes()
    provenance = {'schemaVersion': 1, 'sourceRepository': source_repo, 'sourceCommit': source_commit, 'licenseSpdx': ['GPL-3.0-only'],
        'components': [{'name': 'arex-aniworld', 'origin': source_repo, 'path': 'sources/aniworld', 'licenseSpdx': 'GPL-3.0-only'},
                       {'name': 'arex-sdk', 'origin': source_repo, 'path': 'sdk/rust', 'licenseSpdx': 'GPL-3.0-only'}],
        'localModifications': [], 'compilerVersion': 'rustc 1.95.0 + Binaryen 133', 'sdkVersion': '0.1.0',
        'dependencyLockDigest': a.sha(lock), 'reproducibleBuildCommand': 'bash tools/build_aniworld.sh', 'workflowIdentity': WORKFLOW,
        'moduleDigest': a.sha(module)}
    prov = a.jcs(a.validate(provenance, 'Provenance'))
    manifest = {'schemaVersion': 1, 'extensionId': 'de.aniworld', 'providerId': 'aniworld', 'displayName': 'AniWorld', 'version': version,
        'releaseSequence': release_sequence, 'hostApiMin': 1, 'hostApiMax': 1, 'capabilities': a.ANIWORLD_ROLES.copy(),
        'navigationCapabilities': a.ANIWORLD_NAVIGATION.copy(), 'allowedHosts': ['aniworld.to'],
        'digests': {n: {'sha256': a.sha(d), 'bytes': len(d)} for n, d in [('module', module), ('provenance', prov), ('notice', notice)]},
        'publisherId': PUBLISHER_ID, 'keyId': a.key_id(key), 'sourceRepository': source_repo, 'sourceCommit': source_commit,
        'build': {'toolchainVersion': '1.95.0', 'target': 'wasm32v1-none', 'lockfileDigest': a.sha(lock), 'workflowIdentity': WORKFLOW}}
    return a.assemble_package(module, manifest, provenance, notice, key)

def build_chain(seed: bytes, module: bytes, source_repo: str, source_commit: str, release_sequence: int, version: str,
                origin: str, base_path: str, now: datetime, index_sequence: int):
    """Returns (arex bytes, root envelope, index envelope, pin). base_path is the catalog directory below the origin."""
    keys = derive_keys(seed)
    data, manifest = build_package(module, source_repo, source_commit, release_sequence, version, keys['publisher-1'],
                                   (a.ROOT / 'Cargo.lock').read_bytes())
    ordered = [keys[label] for label in LABELS]
    ids = [a.key_id(k) for k in ordered]
    signed_root = {'schemaVersion': 1, 'repositoryId': REPOSITORY_ID, 'version': 1, 'expiresAt': ROOT_EXPIRES,
        'keys': [{'keyId': a.key_id(k), 'publicKey': a.b64(a.public(k))} for k in ordered],
        'roles': {'root': {'threshold': 2, 'keyIds': ids[:3]}, 'index': {'threshold': 1, 'keyIds': [ids[3]]}},
        'publishers': [{'publisherId': PUBLISHER_ID, 'extensionId': manifest['extensionId'], 'providerId': manifest['providerId'],
            'keyId': ids[4], 'roles': manifest['capabilities'], 'navigation': manifest['navigationCapabilities'],
            'hosts': manifest['allowedHosts'], 'notBefore': PUBLISHER_NOT_BEFORE, 'expiresAt': ROOT_EXPIRES}],
        'revokedKeys': [], 'revokedDigests': []}
    root = a.envelope(signed_root, ordered[:2], 'ROOT')
    pin = {'repositoryId': REPOSITORY_ID, 'initialRootSha256': a.sha(a.jcs(signed_root)), 'distributionOrigins': [origin]}
    entry = {n: manifest[n] for n in ['extensionId', 'providerId', 'displayName', 'navigationCapabilities', 'publisherId', 'keyId',
                                      'version', 'releaseSequence', 'hostApiMin', 'hostApiMax']}
    entry.update({'packageUrl': origin + base_path + '/dist/' + manifest['extensionId'] + '/' + manifest['version'] + '/' + a.sha(data) + '.arex',
        'archiveSha256': a.sha(data), 'archiveBytes': len(data), 'manifestSha256': a.sha(a.jcs(manifest)), 'yanked': False, 'revoked': False})
    index = a.envelope({'schemaVersion': 1, 'repositoryId': REPOSITORY_ID, 'rootVersion': 1, 'sequence': index_sequence,
        'issuedAt': iso(now), 'expiresAt': iso(now + INDEX_VALIDITY), 'entries': [entry]}, [keys['index-1']], 'INDEX')
    a.verify_root(root, pin, now)
    a.verify_index(index, root, pin, now)
    a.verify_package(data, index['signed']['entries'][0], root, now)
    return data, root, index, pin

def write_catalog(output: Path, data: bytes, root: dict, index: dict, version: str):
    entry = index['signed']['entries'][0]
    package = output / 'dist' / 'de.aniworld' / version / (entry['archiveSha256'] + '.arex')
    package.parent.mkdir(parents=True, exist_ok=True)
    package.write_bytes(data)
    (output / 'root.json').write_bytes(a.jcs(root))
    (output / 'index.json').write_bytes(a.jcs(index))
    return package

def fingerprint(root: dict) -> str:
    return a.sha(a.jcs(root['signed']))

def cli():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='cmd', required=True)
    p = sub.add_parser('build')
    p.add_argument('--module', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--source-repository', required=True)
    p.add_argument('--source-commit', required=True)
    p.add_argument('--release-sequence', type=int, required=True)
    p.add_argument('--origin', required=True)
    p.add_argument('--base-path', required=True)
    args = parser.parse_args()
    seed = os.environ.get('PREVIEW_KEY_SEED', '').encode()
    if not seed:
        sys.exit('PREVIEW_KEY_SEED is not set: create the repository secret first (see docs/private-preview.md)')
    now = datetime.now(timezone.utc).replace(microsecond=0)
    version = '1.0.0-preview.' + str(args.release_sequence)
    data, root, index, pin = build_chain(seed, args.module.read_bytes(), args.source_repository, args.source_commit, args.release_sequence,
                                         version, args.origin, args.base_path, now, int(now.timestamp()))
    args.output.mkdir(parents=True, exist_ok=True)
    package = write_catalog(args.output, data, root, index, version)
    # Public values only. No key material is ever printed.
    print(json.dumps({'status': 'PRIVATE_PREVIEW_MANUAL_TRUST', 'repositoryId': pin['repositoryId'], 'rootFingerprintSha256': fingerprint(root),
        'publisherId': PUBLISHER_ID, 'version': version, 'releaseSequence': args.release_sequence, 'indexSequence': index['signed']['sequence'],
        'indexExpiresAt': index['signed']['expiresAt'], 'archiveSha256': a.sha(data), 'package': str(package.relative_to(args.output))}, indent=2))

if __name__ == '__main__':
    cli()
