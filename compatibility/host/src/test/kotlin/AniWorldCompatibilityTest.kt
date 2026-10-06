package com.axiel7.anihyou.release.data.extension

import com.axiel7.anihyou.release.core.extension.*
import java.io.File
import java.nio.file.Files
import java.time.Instant
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.*
import org.junit.Test
import org.junit.Assert.*

class AniWorldCompatibilityTest {
    private val root=File("../../build/aniworld-chain")
    private val inputs=File("../../build/aniworld-inputs")
    private val outputs=File("../../build/aniworld-wire")
    private val now=Instant.parse("2026-09-29T12:00:00Z")
    private fun json(file:File)=ExtensionWireCodec.parseStrictJson(file.readBytes(),4*1024*1024).jsonObject
    private fun pin():AppTrustPin {
        val p=json(File(root,"test-pin.json"));return AppTrustPin(p.getValue("repositoryId").jsonPrimitive.content,p.getValue("initialRootSha256").jsonPrimitive.content,p.getValue("distributionOrigins").jsonArray.map{it.jsonPrimitive.content}.toSet())
    }
    @Test fun realProviderPackageVerifiesInstallsAndRestarts()=runBlocking {
        val p=pin();val trust=ExtensionTrustVerifier(p);val trusted=trust.root(File(root,"root.json").readBytes(),null,now);val index=trust.index(File(root,"index.json").readBytes(),trusted,null,now)
        val entry=index.packages.single();val verifier=ExtensionPackageVerifier(StrictWasmModuleProfileVerifier());val roles=SourceRole.entries.toSet();val hosts=setOf("aniworld.to");val archive=File(root,"aniworld-test.arex")
        val pkg=verifier.verify(archive,entry.binding,trust.publisher(trusted,entry,now),roles,hosts,1,"wasmtime-48.0.3",now)
        assertEquals("AniWorld",pkg.displayName);assertEquals("de.aniworld",pkg.extensionId.value);assertEquals("aniworld",pkg.providerId.value);assertEquals(roles,pkg.grantedRoles);assertEquals(hosts,pkg.grantedHosts)
        val storage=Files.createTempDirectory("aniworld-test-install").toFile()
        try {
            fun store()=ExtensionInstallStore(storage,p,verifier,roles,hosts,1,"wasmtime-48.0.3",smoke={assertArrayEquals(pkg.moduleBytes,it.moduleBytes)})
            val s=store();s.acceptRoot(File(root,"root.json").readBytes(),now);s.acceptIndex(File(root,"index.json").readBytes(),now);s.install(archive,"de.aniworld",now);s.promoteHealthy(now)
            assertEquals(pkg.packageDigest,store().loadUsable(pkg.providerId)!!.packageDigest)
            assertThrows(IllegalArgumentException::class.java){s.install(archive,"de.aniworld",now)}
            val corrupt=File(storage,"corrupt.arex");corrupt.writeBytes(archive.readBytes().also{it[100]=(it[100].toInt() xor 1).toByte()})
            assertThrows(IllegalArgumentException::class.java){verifier.verify(corrupt,entry.binding,trust.publisher(trusted,entry,now),roles,hosts,1,"wasmtime-48.0.3",now)}
            assertEquals(pkg.packageDigest,store().loadUsable(pkg.providerId)!!.packageDigest)
        }finally{storage.deleteRecursively()}
    }
    @Test fun allRealGuestOutputsPassUnmodifiedHostCodecsAndExpectedFacts() {
        val cases=ExtensionWireCodec.parseStrictJson(File(inputs,"cases.json").readBytes(),262144).jsonArray
        for (case in cases) {
            val descriptor=case.jsonObject;val name=descriptor.getValue("name").jsonPrimitive.content;val expected=descriptor.getValue("count").jsonPrimitive.int
            val source=File(inputs,"$name-input.json");val output=File(outputs,"$name-output.json").readBytes()
            if(name.contains("release-plan")){
                val c=ExtensionWireCodec.decodePlanInput(source.readBytes()).context;assertEquals(name,expected,ExtensionWireCodec.decodePlanOutput(output,c).requests.size)
            }else if(name.contains("release-parse")){
                val i=ExtensionWireCodec.decodeParseInput(source.readBytes());val o=ExtensionWireCodec.decodeParseOutput(output,i);assertEquals(name,expected,o.observations.size)
                assertEquals(name,descriptor.getValue("outcomes").jsonArray.map{it.jsonPrimitive.content},o.responseReports.map{it.outcome.name})
                if(descriptor["unknown"]?.jsonPrimitive?.boolean==true)assertTrue(o.observations.all{it.track==ObservationTrack.UNKNOWN})
                o.observations.filter{it.sourceRole==SourceRole.CALENDAR}.forEach{assertEquals(ObservationClaimKind.FORECAST,it.claimKind)}
                o.observations.filter{it.sourceRole==SourceRole.POSTPONEMENT}.forEach{assertNull(it.providerSeriesKey);assertNull(it.parsedTimestamp)}
            }else{
                val value=json(source);val c=if(name.endsWith("-plan"))context(value)else context(value.getValue("context").jsonObject)
                if(name.endsWith("-plan")){assertEquals(name,expected,NavigationWireCodecV1.decodePlan(output,c,setOf("aniworld.to")).requests.size)}
                else{
                    val responses=value.getValue("responses").jsonArray.map{r->val x=r.jsonObject;NavigationResponseEnvelopeV1(x.getValue("requestId").jsonPrimitive.content,ExtensionResponseStatus.valueOf(x.getValue("status").jsonPrimitive.content),x.getValue("httpStatus").jsonPrimitive.int,x.getValue("finalUrl").jsonPrimitive.content,x.getValue("bodyUtf8").jsonPrimitive.content,x.getValue("sourceHash").jsonPrimitive.content)}
                    val parsed=NavigationWireCodecV1.decodeTargets(output,c,responses,setOf("aniworld.to"));assertEquals(name,expected,parsed.targets.size)
                }
            }
        }
    }
    private fun context(x:JsonObject):NavigationContextV1 {
        fun text(k:String)=x.getValue(k).takeUnless{it==JsonNull}?.jsonPrimitive?.content
        return NavigationContextV1(1,ExtensionId.parse(text("extensionId")!!),ProviderId.parse(text("providerId")!!),text("observedAt")!!,NavigationTargetKind.valueOf(text("targetKind")!!),text("targetToken")!!,text("providerSeriesKey")!!,text("providerRouteHint"),text("sourceSeason")?.toInt(),text("providerEpisode"),text("track")?.let{ObservationTrack.valueOf(it)})
    }
}
