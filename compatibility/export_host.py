"""Export unmodified verifiers from one reviewed app HEAD. Verify bytes before use."""
import hashlib,json,re,sys
from pathlib import Path
repo=Path(__file__).resolve().parents[1];host=Path(sys.argv[1])
lock=json.loads((repo/'compatibility/host-source-lock.json').read_text())
out=repo/'compatibility/host/src/main/kotlin';out.mkdir(parents=True,exist_ok=True)
selected=['ExtensionContracts.kt','NavigationContractsV1.kt','ExtensionUpdate.kt','ExtensionPackageVerifier.kt','ExtensionTrust.kt','ExtensionWireCodec.kt','NavigationWireCodecV1.kt','ExtensionInstallStore.kt','ArexZipArchiveReader.kt','StrictWasmModuleProfileVerifier.kt']
for path,digest in lock['files'].items():
    if Path(path).name not in selected and path!='tools/ep02-android/native/src/lib.rs':continue
    data=(host/path).read_bytes()
    if hashlib.sha256(data).hexdigest()!=digest:raise ValueError('host source drift: '+path)
    if Path(path).name in selected:(out/Path(path).name).write_bytes(data)
coordinator=host/'private/release-data/src/main/kotlin/com/axiel7/anihyou/release/data/extension/ExtensionHostCoordinator.kt'
text=coordinator.read_text();start=text.index('class VerifiedExtensionPackage');end=text.index('/**\n * Production implementations',start)
assert hashlib.sha256(coordinator.read_bytes()).hexdigest()==lock['files'][str(coordinator.relative_to(host))], 'repository contract source drift'
imports='package com.axiel7.anihyou.release.data.extension\nimport com.axiel7.anihyou.release.core.extension.*\n'
(out/'RepositoryContracts.kt').write_text(imports+text[start:end])
native=(host/'tools/ep02-android/native/src/lib.rs').read_text();start=native.index('fn build_engine()');end=native.index('\nfn engine()',start)
code='use anyhow::Result;\nuse wasmtime::{Config,Engine,Strategy,WasmFeatures};\nconst MAX_MEMORY_BYTES:usize=32*1024*1024;\n'+native[start:end].replace('fn build_engine()','pub fn build_engine()',1)
dest=repo/'compatibility/native/src/host_engine.rs';dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(code)
