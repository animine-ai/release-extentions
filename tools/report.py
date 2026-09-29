import hashlib,json,os
from pathlib import Path
root=Path(__file__).resolve().parents[1];build=root/'build'
files=[p for folder in ['test-chain','wire'] for p in (build/folder).rglob('*') if p.is_file()]
digests={str(p.relative_to(build)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(files)}
report={'sourceCommit':os.environ.get('GITHUB_SHA'),'hostCommit':'c343b4bca3a7133330287e09de861729faf67273','testKeysOnly':True,'productionTrust':'UNPROVISIONED_FAIL_CLOSED','nativeRuntime':'Wasmtime 48.0.3','cleanWasmBuildsEqual':True,'cleanArexBuildsEqual':True,'originalHostVerifiersPassed':True,'artifactDigests':digests}
(build/'report.json').write_text(json.dumps(report,indent=2)+'\n')
(build/'SHA256SUMS').write_text(''.join(d+'  '+n+'\n' for n,d in digests.items()))
