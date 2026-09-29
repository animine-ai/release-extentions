# Immutable packages

Publish to /dist/<extension-id>/<version>/<archive-sha256>.arex on a separately configured immutable HTTPS distribution service. Never overwrite a digest path. Test CI artifacts are not a production catalog. A mutable GitHub branch/raw URL is not an immutable package origin.
