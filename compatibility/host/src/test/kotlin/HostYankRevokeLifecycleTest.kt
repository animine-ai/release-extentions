package com.axiel7.anihyou.release.data.extension

import com.axiel7.anihyou.release.core.extension.*
import java.io.File
import java.nio.file.Files
import java.time.Instant
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.*
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Contract of the original install store for index-signed withdrawals (planner decision D1).
 *
 * Yank is a distribution withdrawal: the healthy installed package keeps running, also after a restart, but the same
 * digest cannot be installed again, and a strictly higher authenticated index may lift it.
 * Revocation is a security classification: the digest never runs again, whatever a later index says.
 *
 * The pinned host commit in compatibility/host-source-lock.json must contain the D1 install store for this test to
 * pass. If the host changes either rule, this test fails and docs/repository-contract.md must change with it.
 */
class HostYankRevokeLifecycleTest {
    private val dir = File(System.getProperty("arex.diff"), "lifecycle")
    private val now = Instant.parse("2026-09-29T12:00:00Z")
    private val provider = ProviderId.parse("fixture")

    private class Rig(val storage: File, val store: () -> ExtensionInstallStore, val packageFile: File)

    private fun <T> withRig(kind: String, block: suspend (Rig) -> T): T = runBlocking {
        val base = File(dir, "base")
        val p = ExtensionWireCodec.parseStrictJson(File(base, "test-pin.json").readBytes(), 262144).jsonObject
        val pin = AppTrustPin(
            p.getValue("repositoryId").jsonPrimitive.content,
            p.getValue("initialRootSha256").jsonPrimitive.content,
            p.getValue("distributionOrigins").jsonArray.map { it.jsonPrimitive.content }.toSet(),
        )
        val verifier = ExtensionPackageVerifier(StrictWasmModuleProfileVerifier())
        val storage = Files.createTempDirectory("arex-lifecycle-$kind").toFile()
        try {
            fun newStore() = ExtensionInstallStore(
                storage, pin, verifier, setOf(SourceRole.CALENDAR), setOf("example.org"), 1, "wasmtime-48.0.3", smoke = { },
            )
            val packageFile = File(base, "fixture.arex")
            val store = newStore()
            store.acceptRoot(File(base, "root.json").readBytes(), now)
            store.acceptIndex(File(base, "index.json").readBytes(), now)
            store.install(packageFile, "fixture.release", now)
            store.promoteHealthy(now)
            assertNotNull("$kind: installed package must be usable first", store.loadUsable(provider))
            block(Rig(storage, ::newStore, packageFile))
        } finally {
            storage.deleteRecursively()
        }
    }

    private fun accept(rig: Rig, kind: String, name: String) =
        runCatching { rig.store().acceptIndex(File(dir, "$kind/$name").readBytes(), now) }

    @Test fun yankedHealthyPackageKeepsRunningAndCannotBeReinstalledAndHigherIndexLiftsIt() = withRig("yank") { rig ->
        assertTrue("index-signed yank is accepted", accept(rig, "yank", "index-2.json").isSuccess)
        assertNotNull("healthy package keeps running after a yank", rig.store().loadUsable(provider))
        assertNotNull("and after a restart", rig.store().loadUsable(provider))
        val snapshot = rig.store().snapshot()
        assertTrue("a yank is no quarantine", snapshot.quarantinedDigests.isEmpty())
        assertTrue("and no revocation", snapshot.revokedDigests.isEmpty())
        assertFalse(
            "the same digest cannot be installed again",
            runCatching { rig.store().install(rig.packageFile, "fixture.release", now) }.isSuccess,
        )
        assertTrue("a strictly higher authenticated index lifts the pure yank", accept(rig, "yank", "index-3.json").isSuccess)
        assertNotNull("package still runs after the yank is lifted", rig.store().loadUsable(provider))
        assertFalse(
            "a replayed older index cannot bring the yank back or lower the high-water mark",
            accept(rig, "yank", "index-2.json").isSuccess,
        )
    }

    @Test fun revokedPackageNeverRunsAgainEvenWhenALaterIndexClearsTheFlag() = withRig("revoke") { rig ->
        assertTrue("index-signed revocation is accepted", accept(rig, "revoke", "index-2.json").isSuccess)
        assertNull("revoked package stops running", rig.store().loadUsable(provider))
        // Whether the host accepts the clearing index is not the contract; staying blocked is.
        accept(rig, "revoke", "index-3.json")
        assertNull("clearing the flag must not restore the package", rig.store().loadUsable(provider))
        assertNull("still blocked after a restart", rig.store().loadUsable(provider))
        assertFalse(
            "the same digest cannot be reinstalled",
            runCatching { rig.store().install(rig.packageFile, "fixture.release", now) }.isSuccess,
        )
        assertEquals("revocation is recorded permanently", 1, rig.store().snapshot().revokedDigests.size)
    }
}
