# Repository hosting contract (host-enforced)

This is what the unmodified app host requires of a user-added extension repository. Every row is taken from the
host sources pinned in `compatibility/host-source-lock.json` and is either enforced by `tools/arex.py lint-repository`
or exercised by `compatibility/host/src/test/kotlin/DifferentialParityTest.kt`. If the host changes, update the lock
first; this file follows the lock, never the other way round.

Scope: the contract that exists at EP07. It is not a promise about production onboarding, which stays fail closed
(`registry/PRODUCTION-BLOCKED.json`).

## What a user pastes

| Rule | Host behaviour |
|---|---|
| Scheme | `https` only |
| Userinfo, query, fragment | rejected |
| Port | absent or `443` |
| Host | lower-case DNS name with at least one dot, labels 1-63 chars, total <= 253, not an all-digit/dot (IP) literal |
| Path | no `%`, no `\`, no `.`/`..` segment, no `//`; one trailing `/` is trimmed |
| Length / charset | 1-2048 characters, ASCII 0x21-0x7e |
| Origin | `https://<host>` must be in the app pin's `distributionOrigins` |

Consequences for publishers:

- The app derives `<url>/root.json` and `<url>/index.json`. Serve exactly these two names under the pasted URL.
- The pasted URL is not the trust anchor. The origin must already be pinned in the app (`BuildConfig` pin fields).
  A repository on a new domain needs an app release until a generic bootstrap exists.
- Redirects are followed at most 5 times, each target is normalised by the same rules and must again be in the
  pinned origins. A CDN host that is not pinned breaks the repository, also for package downloads.

## Fetch limits

| Document | Maximum bytes | Where |
|---|---|---|
| `root.json` | 65 536 | `FileExtensionSourceRepository` |
| `index.json` | 262 144 | `FileExtensionSourceRepository` |
| `*.arex` package | 8 388 608 | `FileExtensionSourceRepository`, `ExtensionRepositoryTransport` |
| Redirect response body | smaller of the redirect cap and the document cap | transport |

Further transport rules: 10 s per hop, 60 s per request, <= 64 response headers / 16 KiB of header bytes,
`Content-Encoding` must be absent or `identity`, `Content-Length` must be a plain decimal within the cap, 429 and 5xx
are transient (retryable), any other non-2xx is a hard failure. A body over the cap fails the whole fetch; there is no
partial acceptance.

## Catalog capacity (signed content)

| Item | Cap |
|---|---|
| index `entries` | 256 |
| root `publishers` | 32 |
| root `keys` / `revokedKeys` | 32 / 32 |
| root `revokedDigests` | 256 |
| `hosts` per publisher scope | 32 |
| envelope `signatures` | 32 |
| JSON nesting | 16 levels |
| JSON array entries | 4096 |
| JSON object fields | 256 |

Overflowing any of these rejects the whole document. There is no truncation, so leave headroom:
`lint-repository` warns at 80 % of the index byte cap and of the entry cap.

## Whole-catalog effects publishers must plan for

1. **One bad scope rejects the entire index.** The host checks the publisher scope of every entry when it accepts an
   index. A scope that is expired or not yet valid for any single entry rejects all entries, including unrelated
   extensions. Keep every scope valid for the full lifetime of any index that references it; `lint-repository` warns when
   a scope ends before the index does. Evidence: `scopes.json` cases in `DifferentialParityTest`.
2. **Index validity is at most 7 days** (`issuedAt` may be up to 10 minutes in the future). A repository that does
   not re-sign before expiry stops delivering updates; already installed packages stay usable.
3. **`yanked` and `revoked` are one-way on the device.** A digest that was ever seen as revoked or yanked is kept in
   device state. Re-publishing the same digest as healthy later does not restore it. Cut a new release with a higher
   `releaseSequence` instead.
4. **Revocation lives in two places.** The index entry flag and the root `revokedDigests`/`revokedKeys`. The root list is
   append-only across rotations; plan the 256-digest cap.
5. **Release sequence is per extension and strictly increasing**; entries are ordered by `(extensionId, releaseSequence)`.
6. **Package URLs are immutable bindings:** `https://<pinned origin>/.../<archiveSha256>.arex`, no query, no `%`, no `..`.

## Things the app build fixes, not the repository

| Fixed in the app build (`BuildConfig` / module constants) | Effect on a repository |
|---|---|
| Pinned root digest and `repositoryId` | A new repository identity needs an app release |
| Pinned `distributionOrigins` | Domain move needs an app release |
| Approved-authority publisher and `signingKeyId` | Key rotation of the release authority key needs an app release even when the root rotates correctly |
| Per-provider fuel budget (for `de.aniworld`: 25 000 000) | A guest that needs more fuel fails at runtime, not at install |

## Tooling

```
python3 tools/arex.py lint-repository --url https://packages.example.org/extensions \
    --root root.json --index index.json --pin app-pin.json [--strict]
```

Errors are conditions the host rejects. Warnings are conditions that will make a later fetch fail or that remove
headroom. `--strict` turns warnings into a non-zero exit. The lint does not replace `verify-package`; it checks the
repository documents and address, not archive contents.

CI generates hostile JSON, display-name, archive and scope vectors (`tools/differential_vectors.py`) and replays the same
bytes through the original host code. The tool may only be stricter than the host, and the list of cases where it is
stricter is exact, so a drift in either direction fails CI.
