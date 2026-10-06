"""Require retained, exact-guest Android EP04 evidence. No network access."""
import hashlib
import json
import re
import sys
from pathlib import Path

report_path, module_path, source_commit, cases_path = sys.argv[1:]
report = json.loads(Path(report_path).read_text())
assert report['passed'] is True and report['api'] == 35
assert report['runtimePin'] == 'Wasmtime 48.0.3 LTS / Cranelift'
assert report['functional']['status'] == 'PASS'
checks = report['functional']['checks']
assert checks['fixtureOnlyNoFallback'] is True
proof = checks['ep04AniWorld']  # Missing optional guest proof must fail this EP04 gate.
assert proof['sourceCommit'] == source_commit
assert proof['moduleDigest'] == hashlib.sha256(Path(module_path).read_bytes()).hexdigest()
assert re.fullmatch('[0-9a-f]{64}', proof['packageDigest'])
assert proof['generatedVectorCount'] == len(json.loads(Path(cases_path).read_text()))
assert set(proof['releasePlanRoles']) == {'CALENDAR', 'RECENT', 'POSTPONEMENT', 'DIRECT'}
assert [proof[k] for k in ['calendarObservations', 'recentObservations', 'postponementObservations', 'directObservations']] == [2, 2, 2, 1]
assert proof['overviewNavigationUrl'] == 'https://aniworld.to/anime/stream/fixture-series'
assert proof['episodeNavigationUrl'] == 'https://aniworld.to/anime/stream/fixture-series/staffel-1/episode-1'
assert proof['moduleDigestMismatchRejected'] == 'INVALID_INPUT'
assert proof['responseBodyDigestMismatchRejected'] == 'HOST_VALIDATION_FAILED'
assert proof['failClosedVectors'] == {
    'bot-release-parse': 'FAILURE', 'truncated-release-parse': 'FAILURE',
    'malformed-date-release-parse': 'PARTIAL', 'unknown-track-release-parse': 'PARTIAL',
    'missing-direct-release-parse': 'FAILURE', 'canonical-mismatch-episode-parse': 'NO_TARGET',
    'missing-episode-parse': 'NO_TARGET', 'unavailable-dub-episode-parse': 'NO_TARGET',
}
assert proof['isolatedServiceNoInternet'] is True
assert all(proof['realGuestCancellation'][k] is True for k in ['enteredIsolatedService', 'coroutineCancelled', 'lateResultFenced', 'normalCallRecovered'])
assert proof['realGuestCacheHit'] == {'cacheHit': True, 'outputStable': True}
assert proof['realGuestFuelAbort'] == {'fuel': 1, 'runtimeResult': 'FAILURE', 'errorCode': 'TRAP', 'normalCallRecovered': True}
restart = proof['realGuestRestartRecovery']
assert restart['normalCallRecovered'] is True and restart['newPid'] != restart['oldPid']
assert restart['newGeneration'] > restart['oldGeneration']
receipt = proof['realGuestCoordinator']
assert receipt['completed'] is True and receipt['observationCount'] == 7
assert receipt['receiptGenerationMatchesRequest'] is True
assert receipt['receiptGenerationId'] == receipt['requestGenerationId']
assert receipt['runtimeServiceGenerationBefore'] == receipt['runtimeServiceGenerationAfter']
assert receipt['productionNetworkLedgerUsed'] is False
canary = checks['ep05Canary']
assert canary['testTrustOnly'] is True and canary['productionPublication'] is False
assert canary['moduleDigest'] == proof['moduleDigest']
assert re.fullmatch('[0-9a-f]{64}', canary['packageDigest'])
assert canary['observationCount'] == 7 and canary['evidenceCount'] == 5
assert all(canary[k] is True for k in ['realProductionHttpsTransport', 'dnsBoundTlsSocket',
    'isolatedRealGuest', 'authorityRequiresExactHostTuple', 'calendarForecastOnly',
    'calendarWallTimeWithoutInventedTimezone', 'unknownTrackNoAuthority',
    'unboundPostponementNoAuthority', 'roomShadowCommitted', 'idempotentWorkRetry'])
assert len(canary['provenance']) == 4
assert all(p['destination'] == '8.8.8.8' and p['httpStatus'] == 200 for p in canary['provenance'])
assert len(canary['sourceHealth']) == 4 and all(h['status'] == 'HEALTHY' for h in canary['sourceHealth'])
assert {n['kind'] for n in canary['navigation']} == {'OVERVIEW', 'EPISODE'}
assert canary['authorityDecision']['underlyingPhase'] == 'RELEASED'
assert canary['authorityDecision']['authority'] == 'ANIWORLD'
print('EP04 exact real-guest Android evidence verified')
