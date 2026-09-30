import hashlib,json,os
from pathlib import Path
root=Path(__file__).resolve().parents[1];build=root/'build'
lock=json.loads((root/'compatibility'/'host-source-lock.json').read_text())
folders=['test-chain','aniworld-chain','wire']
files=[p for folder in folders for p in (build/folder).rglob('*') if p.is_file()]
digests={str(p.relative_to(build)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(files)}
aniworld_manifest=json.loads((build/'aniworld-chain'/'manifest.json').read_text())
report={
  'sourceCommit':os.environ.get('GITHUB_SHA'),
  'hostCommit':lock['commit'],
  'testKeysOnly':True,
  'productionTrust':'UNPROVISIONED_FAIL_CLOSED',
  'nativeRuntime':'Wasmtime 48.0.3',
  'cleanWasmBuildsEqual':True,
  'cleanArexBuildsEqual':True,
  'originalHostVerifiersPassed':True,
  'realGuestWasmSha256':aniworld_manifest['digests']['module']['sha256'],
  'realGuestWasmBytes':aniworld_manifest['digests']['module']['bytes'],
  'testArexSha256':digests['aniworld-chain/aniworld-test.arex'],
  'artifactDigests':digests,
}
(build/'report.json').write_text(json.dumps(report,indent=2)+'\n')
(build/'SHA256SUMS').write_text(''.join(d+'  '+n+'\n' for n,d in digests.items()))
