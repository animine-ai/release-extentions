# Release Extensions, ABI v1

Versioned SDK, deterministic WASM fixtures and offline distribution tools for the
accepted AniHyou host at `c343b4bca3a7133330287e09de861729faf67273`.

## User-added sources

A fresh app has zero configured repositories. There is no opt-in prompt, bundled
catalog or package, and no repository URL/publisher-specific binding in app defaults.
Users choose **Repo hinzufügen**, paste a URL, then see verified extension metadata
and activate extensions. The first package downloads on activation. At startup a
background task checks signed metadata and monotone versions/digests. Unchanged
archives remain local. Updated archives are staged, verified and atomically promoted,
with last-known-good rollback on failure and revocation taking precedence.

This repository provides the distribution layer. The generic app source-add UI,
per-user source storage and update scheduling are a separate app-stage dependency.
No implementation of that UI is claimed in EP03.

## Production trust status

**UNPROVISIONED, FAIL CLOSED.** `registry/PRODUCTION-BLOCKED.json` is a status document,
not a root or catalog. No production root, signing key, catalog or app pin is fabricated.
The one-URL flow requires a reviewed generic bootstrap that authenticates a repository
root without embedding this repository in the app. A fetched key does not authenticate
itself. The accepted host currently requires an independently authenticated initial
root digest and origins; extending its bootstrap is an explicit future security task.

Initial roots require 2 of 3 distinct Ed25519 root signers. Index signing is delegated,
as are publisher scopes, roles, navigation capabilities and exact hosts. Root rotation
is sequential and meets both old and new thresholds. Signed index sequences and
digests prevent rollback/equivocation, expiry limits stale metadata, and known
revocation overrides LKG. Installation grants no release Authority.

All deterministic seeds in `tools/arex.py` are **PUBLIC TEST ONLY**. Never use them in
production. The protected-release workflow is intentionally fail-closed until offline
key custody, public keys, generic bootstrap and immutable storage are reviewed.
An environment name alone does not create protection: required reviewers and branch
rules must be configured before enabling a future release workflow.

## Runtime and package

The guest is wasm32 core, no WASI, clocks, randomness, guest networking or filesystem.
The host fetches bounded HTTPS bytes and invokes isolated, bounded plan/parse calls.
One package can implement release roles and optional overview/episode navigation.
Navigation never grants release Authority. Provider behavior belongs to extensions.

`.arex` is a strict root-only ZIP containing exactly `manifest.json`, `module.wasm`,
`provenance.json`, `NOTICE`, `package.sig`. Ed25519 signs
`AREX-PACKAGE-V1\n` followed by RFC8785/JCS manifest bytes. The manifest binds module,
provenance and notice digest/size; the signed index binds archive URL, digest, size and
canonical manifest digest. Root/index signatures are inside `{signed, signatures}`
envelopes, matching the accepted host. There are no separate detached root.sig/index.sig
files in this wire format.

`sdk/schema/v1.schema.json` defines the frozen structures; entrypoint schemas reference
it. JSON Schema describes the structural contract. Strict UTF-8 byte caps, duplicate
keys, cross-field identities, coordinate binding and provenance are additionally
validated by the SDK/tools and the original host. Signed metadata has no float fields;
the canonicalizer accepts only JCS-safe integer/string/bool/null containers and rejects
floats rather than silently rounding them. This is the exact signed-schema subset.

## Build and test

Install Python 3.12, Rust 1.95.0 with wasm32v1-none, and the pinned dependencies:

```bash
python3 -m pip install -r requirements.txt
python3 -m unittest discover -s tests -v
cargo +1.95.0 test --locked --workspace
bash tools/build_fixture.sh build/fixture
python3 tools/arex.py test-chain --module build/fixture/wasm32v1-none/release/arex_fixture.wasm --output build/test-chain --source-repository https://github.com/YOUR_ACCOUNT/YOUR_REPO --source-commit FULL_SOURCE_COMMIT
python3 tools/arex.py verify-chain build/test-chain
```

CI builds the fixture twice into clean directories and compares WASM and `.arex` bytes.
It checks out the original accepted host by full SHA, verifies its source checksums,
compiles its actual Kotlin package/root/index/installer/codec verifiers and exports its
exact Wasmtime 48.0.3 feature configuration. That native engine executes every fixture
operation twice; its real outputs enter the host codecs. CI performs no live AniWorld
requests. Actions are pinned to full commit SHAs. CI reports and the test fixture are
uploaded as `ep03-fixture-evidence`; they are not production release artifacts.

## SDK and new extensions

`sdk/rust` is no_std + alloc, with serde typed wire DTOs, explicit required nullable
fields, bounded encoding/decoding, owned allocation/free and diagnostic helpers.
`sources/fixture` demonstrates all four operation exports and a bounded per-instance
arena. Authors must validate their operation-specific semantics, use fresh instances,
retain no guest state between plan and parse and return bounded failures.

To add a provider, create a crate, implement the fixed ABI and deterministic fixtures,
declare minimal release/navigation/host grants, supply source/license/build provenance,
obtain a delegated publisher scope and build with `tools/arex.py build --help`. Verify
with the pinned native profile and original host before publishing. Use externally
protected ephemeral key files or a reviewed external signer, not repository secrets
printed in logs. Public root/index metadata can be prepared with `prepare-envelope`,
signed by independent keys with `sign-envelope`, and verified with `verify-root` and
`verify-package`. Detached signatures are not used.

`sources/aniworld` is **EP04 skeleton only**. Its allowed-host/publisher/key fields are
deliberately unset and it cannot produce a publishable package. There are no live
selectors, provider paths, scraping logic or production semantics here.

## Immutable distribution

Use a separately configured HTTPS origin with write-once paths:
`/dist/<extension-id>/<version>/<archive-sha256>.arex`.
Never overwrite one digest path. Keep old versions for LKG unless revoked. A mutable
raw GitHub branch URL is unsuitable as an immutable package path. Repository URLs,
source commits and distribution origins are supplied as release inputs, so renaming
the GitHub repository requires no app change. See `docs/distribution.md`.

## License and provenance

GPL-3.0-only. The export normalizer originates from the accepted host's
`tools/ep02-android/normalize-fixture-wasm.py`. Contract shapes and compatibility sources
are bound to that same public host commit. Original host verifiers are fetched and
compiled during CI, never maintained as a divergent verifier copy. The native lock
comes from accepted EP02 artifact 11022664133, ZIP SHA-256
`357fd456e3984d65eb7bef6daa11b73f995762ff405ca4329b5519065224d48d`.
