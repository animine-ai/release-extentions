# Private preview publication (temporary)

Master decision of 2026-10-04: for a small private user group the app may use a concretely added source after the user explicitly
confirmed the risk (manually accepted first trust, **not** independent verification). This directory's production path
(`release.yml`, `registry/PRODUCTION-BLOCKED.json`) is untouched and stays blocked.

## What the workflow does

`.github/workflows/private-preview.yml` builds the real `de.aniworld` guest twice (the builds must be byte-identical), signs a
normal AREX v1 chain (2-of-3 root, index key, delegated publisher key for `de.aniworld`/`aniworld`, host `aniworld.to` only) and
publishes **only public files** to the branch `private-preview-catalog`:

    catalog/root.json     fixed content, so its fingerprint never changes
    catalog/index.json    re-signed on every run, valid for six days (the verifier caps an index at seven)
    catalog/dist/de.aniworld/<version>/<sha256>.arex   immutable, digest-addressed

Source URL for the app: `https://raw.githubusercontent.com/animine-ai/release-extentions/private-preview-catalog/catalog`

## The one input you must provide

Create the repository secret **`PREVIEW_KEY_SEED`** (Settings > Secrets and variables > Actions): 64 random characters, from a
password manager, no words. All five private keys are derived from it inside the job (`tools/private_preview.py`, HMAC-SHA256).
Nothing derived from it is ever written to a file, printed, or uploaded. Keep the value: it is the only custody item. If it is lost
or changed, the identity changes and every user has to accept the new root.

This is **not** independent three-party custody. The same operator holds every key. The secret is visible to anyone who can
write workflows in this repository.

## Operating rules

- Do not rotate the seed to "refresh" the identity; refresh the index by re-running the workflow (at least every six days).
- Scheduled runs only fire when the workflow file is on the default branch; until then re-run it by hand or push to
  `ep07/private-preview`.
- Revocation and rollback use the existing root/index mechanisms, not new ones.
- To return to the independently verified path later, publish a production root through the protected release workflow; the app
  keeps asking again for any source whose acceptance is removed.
