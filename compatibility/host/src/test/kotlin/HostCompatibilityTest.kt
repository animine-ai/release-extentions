package com.axiel7.anihyou.release.data.extension
import com.axiel7.anihyou.release.core.extension.*
import java.io.File
import java.nio.file.Files
import java.time.Instant
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.*
import org.junit.Test
import org.junit.Assert.*

class HostCompatibilityTest {
    private val directory=File(System.getProperty("arex.artifacts"))
    private val now=Instant.parse("2026-09-29T12:00:00Z")
    private fun bytes(name:String)=File(directory,name).readBytes()
    private fun json(data:ByteArray)=ExtensionWireCodec.parseStrictJson(data,262144).jsonObject
    private fun pin():AppTrustPin {
        val p=json(bytes("test-pin.json"));return AppTrustPin(p.getValue("repositoryId").jsonPrimitive.content,p.getValue("initialRootSha256").jsonPrimitive.content,p.getValue("distributionOrigins").jsonArray.map{it.jsonPrimitive.content}.toSet())
    }
    @Test fun originalHostVerifiesPackageAndInstallLifecycle()=runBlocking {
        val p=pin();val trust=ExtensionTrustVerifier(p);val root=trust.root(bytes("root.json"),null,now);val index=trust.index(bytes("index.json"),root,null,now)
        val item=index.packages.single();val verifier=ExtensionPackageVerifier(StrictWasmModuleProfileVerifier())
        val packageFile=File(directory,"fixture.arex");val pkg=verifier.verify(packageFile,item.binding,trust.publisher(root,item,now),setOf(SourceRole.CALENDAR),setOf("example.org"),1,"wasmtime-48.0.3",now)
        assertEquals("Fixture Provider",pkg.displayName);assertEquals(setOf(NavigationCapability.OVERVIEW_NAVIGATION,NavigationCapability.EPISODE_NAVIGATION),pkg.navigationCapabilities)
        val storage=Files.createTempDirectory("arex-host-compat").toFile()
        try {
            fun newStore()=ExtensionInstallStore(storage,p,verifier,setOf(SourceRole.CALENDAR),setOf("example.org"),1,"wasmtime-48.0.3",smoke={assertArrayEquals(pkg.moduleBytes,it.moduleBytes)})
            val store=newStore();store.acceptRoot(bytes("root.json"),now);store.acceptIndex(bytes("index.json"),now);store.install(packageFile,"fixture.release",now);store.promoteHealthy(now)
            assertEquals(pkg.packageDigest,store.loadUsable(ProviderId.parse("fixture"))!!.packageDigest)
            assertEquals(pkg.packageDigest,newStore().loadUsable(ProviderId.parse("fixture"))!!.packageDigest)
            // Same signed index is harmless; an unchanged package is not installable again.
            store.acceptIndex(bytes("index.json"),now)
            assertThrows(IllegalArgumentException::class.java){store.install(packageFile,"fixture.release",now)}
            val tampered=File(storage,"bad.arex");tampered.writeBytes(packageFile.readBytes().also{it[100]=(it[100].toInt() xor 1).toByte()})
            assertThrows(IllegalArgumentException::class.java){verifier.verify(tampered,item.binding,trust.publisher(root,item,now),setOf(SourceRole.CALENDAR),setOf("example.org"),1,"wasmtime-48.0.3",now)}
        } finally {storage.deleteRecursively()}
    }
    @Test fun sharedTrustVectorsPassOriginalVerifier() {
        val trust=ExtensionTrustVerifier(pin());val root=trust.root(bytes("root.json"),null,now);val index=trust.index(bytes("index.json"),root,null,now)
        val vectors=ExtensionWireCodec.parseStrictJson(bytes("trust-vectors.json"),262144).jsonArray
        for(value in vectors){val v=value.jsonObject;val kind=v.getValue("kind").jsonPrimitive.content;val env=v.getValue("envelope").toString().toByteArray();val prev=v.getValue("previous").jsonPrimitive.boolean;val valid=v.getValue("valid").jsonPrimitive.boolean
            val passed=runCatching {when(kind){"root"->trust.root(env,if(prev)root else null,now);"index"->trust.index(env,root,if(prev)index else null,now);"blocked-package"->{val blocked=trust.index(env,root,index,now);trust.publisher(root,blocked.packages.single(),now)};else->error("vector kind")}}.isSuccess
            assertEquals(v.getValue("name").jsonPrimitive.content,valid,passed)
        }
    }
    @Test fun actualWasmOutputsPassOriginalReleaseAndNavigationCodecs() {
        val inputs=File(System.getProperty("arex.inputs"));val output=File(System.getProperty("arex.wire"))
        val context=ExtensionWireCodec.decodePlanInput(File(inputs,"release-plan-input.json").readBytes()).context
        val plan=ExtensionWireCodec.decodePlanOutput(File(output,"release-plan-output.json").readBytes(),context);assertEquals(1,plan.requests.size)
        val parse=ExtensionWireCodec.decodeParseInput(File(inputs,"release-parse-input.json").readBytes())
        val result=ExtensionWireCodec.decodeParseOutput(File(output,"release-parse-output.json").readBytes(),parse);assertEquals(1,result.observations.size);assertEquals(ExtensionReportOutcome.SUCCESS,result.responseReports.single().outcome)
        for (kind in NavigationTargetKind.entries) {
            val prefix=kind.name.lowercase();val c=NavigationContextV1(1,ExtensionId.parse("fixture.release"),ProviderId.parse("fixture"),now.toString(),kind,"t1","series-1",null,2,if(kind==NavigationTargetKind.EPISODE)"15" else null,if(kind==NavigationTargetKind.EPISODE)ObservationTrack.DE_SUB else null)
            val nav=NavigationWireCodecV1.decodePlan(File(output,"$prefix-plan-output.json").readBytes(),c,setOf("example.org"));assertEquals(1,nav.requests.size)
            val r=json(File(inputs,"$prefix-parse-input.json").readBytes()).getValue("responses").jsonArray.single().jsonObject
            val response=NavigationResponseEnvelopeV1("nav-1",ExtensionResponseStatus.OK,200,"https://example.org/nav-source","fixture-nav-v1",r.getValue("sourceHash").jsonPrimitive.content)
            val target=NavigationWireCodecV1.decodeTargets(File(output,"$prefix-parse-output.json").readBytes(),c,listOf(response),setOf("example.org"));assertEquals(kind,target.targets.single().targetKind)
        }
    }
}
