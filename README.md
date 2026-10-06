# AniHyou Extensions

AniWorld 1.0.0 is the first stable semantic version of the current extension.

Source URL:
https://raw.githubusercontent.com/animine-ai/release-extentions/catalog

The source offers signed AniWorld releases for German subtitles and dubbing, calendars,
recent releases and postponements. Existing installations using the previous catalog URL
continue to receive the same stable update. No reinstall or matching reset is required.

The operator-held signing identity is unchanged. Manual source acceptance is still required;
a stable version number does not claim independent identity verification or separate key custody.
The historical repository/publisher IDs and key derivation remain internal compatibility identifiers.

## Publication

The stable source branch is `stable`; version is read from `sources/aniworld/VERSION`.
The default-branch publication workflow checks out that branch explicitly and refreshes the index daily.
The index is valid for six days. Each publication builds the module twice, compares the bytes,
runs workspace and catalog tests, and verifies the AREX signatures/digests before upload.
A previously published stable version cannot be replaced with different package bytes.
To change extension code, increment VERSION before publication.

The historical `private-preview-catalog/catalog` URL remains an update alias. New installations
should add the clean source URL above. Do not add both URLs to the same installation.
The protected independently provisioned production-signing gate is unchanged.
