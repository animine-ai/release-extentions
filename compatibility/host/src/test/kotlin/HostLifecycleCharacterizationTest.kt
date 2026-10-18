package com.axiel7.anihyou.release.data.extension

import com.axiel7.anihyou.release.core.extension.*
import java.io.File
import java.nio.file.Files
import java.time.Instant
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.*
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

/**
 * Characterization of the original install store: an index-signed yank or revoke of the active package is one-way
 * on the device. A later index that clears the flag does not make the package usable again, not after a restart, and
 * the same digest cannot be reinstalled. docs/repository-contract.md tells publishers to cut a higher releaseSequence.
 * If the host ever changes this on purpose, this test fails and that document must change with it.
 */
class HostLifecycleCharacterizationTest {
    private val dir = File(System.getProperty("arex.diff"), "lifecycle")
    private val now = Instant.parse("2026-09-29T12:00:00Z")
    private val provider = ProviderId.parse("fixture")

    private fun scenario(kind: String) = runBlocking {
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

            store.acceptIndex(File(dir, "$kind/index-2.json").readBytes(), now)
            assertNull("$kind: flagged package must stop being usable", store.loadUsable(provider))

            // Whether the host accepts or rejects the clearing index is not the contract; staying unusable is.
            runCatching { store.acceptIndex(File(dir, "$kind/index-3.json").readBytes(), now) }
            assertNull("$kind: clearing the flag must not restore the package", store.loadUsable(provider))

            val restarted = newStore()
            assertNull("$kind: still unusable after restart", restarted.loadUsable(provider))
            assertFalse(
                "$kind: the same digest must not be reinstallable",
                runCatching { restarted.install(packageFile, "fixture.release", now) }.isSuccess,
            )
        } finally {
            storage.deleteRecursively()
        }
    }

    @Test fun indexYankIsOneWay() = scenario("yank")

    @Test fun indexRevokeIsOneWay() = scenario("revoke")
}
