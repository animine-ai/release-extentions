# Frozen v1 schema export

Definitions come from the accepted host DTOs, package/trust readers and Navigation v1
at c343b4bca3a7133330287e09de861729faf67273. All object fields are required; nullable
fields are explicit null. Unknown fields require a later negotiated schema.

Structural JSON Schema cannot express every runtime invariant. The host remains
normative for UTF-8 byte caps, safe JSON, identifier/URL semantics, authenticated tuple
equality, scope intersections, response hashes, per-role budgets and navigation
coordinates. Shared generated inputs and real WASM outputs are passed through both
schemas and the exact original host codecs in CI. Source-lock verification prevents
quiet host drift. There is no live provider parser in this export.
