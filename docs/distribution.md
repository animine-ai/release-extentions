# Publication and trust boundary

The app ships a generic extension host with no repository source configured. Repository
addition remains explicit and generic. A pasted source URL downloads metadata, not an
automatically trusted signing key. The required generic signing authority or equivalent
independent root authentication has not been provisioned. The accepted per-repository
host pin API cannot directly implement URL-only onboarding without this additional
design. Production installation therefore remains fail-closed.

## Required production inputs

1. Reviewed generic trust-bootstrap design compatible with user-added sources.
2. Three independent offline Ed25519 root custodians, 2-of-3 threshold.
3. Separately delegated index and scoped publisher public keys.
4. Public root bytes, reviewable fingerprint and protected signing ceremony.
5. Exact HTTPS immutable package origin, write-once object storage, backups and LKG
   retention/revocation policy. Production metadata publication must not overwrite
   digest-addressed archives; storage-level retention is required.
6. Protected environment/reviewer/branch controls and an external signing integration.

No private production key is generated, stored in Git or logged by EP03. Ordinary CI
uses public deterministic test seeds. The production status document deliberately does
not masquerade as root.json/index.json.

## Background refresh target

Only user-added sources are checked. Coalesce startup checks and run off the UI thread.
Validate root/index signatures, expiry, revocations and sequence high-water state before
comparing installed package digests/releaseSequence. Equal index sequence is accepted
only for the identical signed digest. Fetch a package only for first activation or a
verified newer installed-extension release. Stage, verify profile/compatibility and
smoke behavior, then atomically promote. Preserve LKG on network/install failure; never
restore revoked content. A source without an activated package contributes no release
Authority. No provider-specific default or source suggestion belongs in the clean app.

This scheduling/onboarding UI is outside EP03's distribution repository and remains an
app-stage implementation dependency, not a claimed feature in the current APK.
