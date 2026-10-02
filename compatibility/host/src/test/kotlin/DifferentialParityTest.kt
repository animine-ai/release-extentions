package com.axiel7.anihyou.release.data.extension

import com.axiel7.anihyou.release.core.extension.*
import java.io.File
import java.time.Instant
import java.util.Base64
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.*
import org.erdtman.jcs.JsonCanonicalizer
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Differential parity between the publisher tool (tools/arex.py) and the unmodified host verifiers.
 *
 * tools/differential_vectors.py writes hostile inputs plus the tool's own verdict. This test replays the same bytes
 * through the original host code. The safety invariant is one-directional:
 *
 *  - the tool accepting what the host rejects is a failure (a publisher could sign a release no device installs, or
 *    two canonicalisers would produce different signed bytes);
 *  - the tool rejecting what the host accepts is allowed only for the named cases below (the tool is a lint gate and
 *    is intentionally stricter), and the list must stay exact so that drift in either direction is noticed.
 */
class DifferentialParityTest {
    private val dir = File(System.getProperty("arex.diff"))
    private val now = Instant.parse("2026-09-29T12:00:00Z")

    private val toolStricterJson = setOf("float-1.0", "float-exp", "float-half", "over-safe", "huge-int")
    private val toolStricterArchives = setOf("extra-field")
    private val toolStricterNames = emptySet<String>()

    private fun hex(bytes: ByteArray) = bytes.joinToString("") { "%02x".format(it) }
    private fun array(name: String) = Json.parseToJsonElement(File(dir, name).readText()).jsonArray

    private fun pin(directory: File): AppTrustPin {
        val p = ExtensionWireCodec.parseStrictJson(File(directory, "test-pin.json").readBytes(), 262144).jsonObject
        return AppTrustPin(
            p.getValue("repositoryId").jsonPrimitive.content,
            p.getValue("initialRootSha256").jsonPrimitive.content,
            p.getValue("distributionOrigins").jsonArray.map { it.jsonPrimitive.content }.toSet(),
        )
    }

    private fun hostPackageVerdict(directory: File): Boolean = runCatching {
        runBlocking {
            val trust = ExtensionTrustVerifier(pin(directory))
            val root = trust.root(File(directory, "root.json").readBytes(), null, now)
            val index = trust.index(File(directory, "index.json").readBytes(), root, null, now)
            val item = index.packages.single()
            ExtensionPackageVerifier(StrictWasmModuleProfileVerifier()).verify(
                File(directory, "fixture.arex"), item.binding, trust.publisher(root, item, now),
                setOf(SourceRole.CALENDAR), setOf("example.org"), 1, "wasmtime-48.0.3", now,
            )
        }
    }.isSuccess

    private fun hostCatalogVerdict(directory: File): Boolean = runCatching {
        val trust = ExtensionTrustVerifier(pin(directory))
        val root = trust.root(File(directory, "root.json").readBytes(), null, now)
        trust.index(File(directory, "index.json").readBytes(), root, null, now)
    }.isSuccess

    private fun compare(file: String, allowedToolStricter: Set<String>, host: (File) -> Boolean) {
        val toolAcceptsHostRejects = mutableListOf<String>()
        val toolStricter = mutableSetOf<String>()
        var compared = 0
        for (element in array(file)) {
            val o = element.jsonObject
            val name = o.getValue("name").jsonPrimitive.content
            val directory = File(dir, o.getValue("dir").jsonPrimitive.content)
            val tool = o.getValue("pyAccept").jsonPrimitive.boolean
            // A case the tool refused to even build is a tool rejection; the host sees no artifact for it.
            val hostAccepts = directory.isDirectory && host(directory)
            compared++
            if (tool && !hostAccepts) toolAcceptsHostRejects += name
            if (!tool && hostAccepts) toolStricter += name
        }
        assertTrue("$file: no vectors were compared", compared > 0)
        assertEquals("$file: tool accepts what the host rejects", emptyList<String>(), toolAcceptsHostRejects)
        assertEquals("$file: allow-list of tool-stricter cases drifted", allowedToolStricter, toolStricter)
    }

    @Test fun strictJsonAndCanonicalBytesAgree() {
        val toolAcceptsHostRejects = mutableListOf<String>()
        val toolStricter = mutableSetOf<String>()
        val canonicalDiffers = mutableListOf<String>()
        val vectors = array("json-vectors.json")
        assertTrue("no JSON vectors", vectors.size > 50)
        for (element in vectors) {
            val o = element.jsonObject
            val name = o.getValue("name").jsonPrimitive.content
            val bytes = Base64.getDecoder().decode(o.getValue("b64").jsonPrimitive.content)
            val tool = o.getValue("pyAccept").jsonPrimitive.boolean
            val toolCanonical = o.getValue("pyCanon").jsonPrimitive.contentOrNull
            val hostCanonical: String? = runCatching {
                val parsed = ExtensionWireCodec.parseStrictJson(bytes, 262144)
                if (parsed is JsonObject) {
                    hex(JsonCanonicalizer(parsed.toString()).encodedString.toByteArray(Charsets.UTF_8))
                } else {
                    null
                }
            }.getOrNull()
            val hostAccepts = hostCanonical != null
            if (tool && !hostAccepts) toolAcceptsHostRejects += name
            if (!tool && hostAccepts) toolStricter += name
            if (tool && hostAccepts && toolCanonical != hostCanonical) canonicalDiffers += name
        }
        assertEquals("tool accepts JSON the host rejects", emptyList<String>(), toolAcceptsHostRejects)
        assertEquals("tool and host signed bytes differ", emptyList<String>(), canonicalDiffers)
        assertEquals("allow-list of tool-stricter JSON cases drifted", toolStricterJson, toolStricter)
    }

    @Test fun displayNamesAgree() = compare("names.json", toolStricterNames, ::hostPackageVerdict)

    @Test fun archiveMutationsAgree() = compare("archives.json", toolStricterArchives, ::hostPackageVerdict)

    /** One out-of-window publisher scope rejects the whole index on the host; the tool must say the same. */
    @Test fun publisherScopeWindowsAgree() = compare("scopes.json", emptySet(), ::hostCatalogVerdict)
}
