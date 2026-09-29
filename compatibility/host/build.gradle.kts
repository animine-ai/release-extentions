plugins { kotlin("jvm") version "2.4.20" }
repositories { mavenCentral() }
kotlin { jvmToolchain(17) }
dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.11.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.11.0")
    implementation("org.bouncycastle:bcprov-jdk18on:1.86")
    implementation("io.github.erdtman:java-json-canonicalization:1.1")
    testImplementation("junit:junit:4.13.2")
}
tasks.test {
    systemProperty("arex.artifacts", file("../../build/test-chain").absolutePath)
    systemProperty("arex.wire", file("../../build/wire").absolutePath)
    systemProperty("arex.inputs", file("../../fixtures/wire").absolutePath)
    testLogging { events("passed","skipped","failed") }
}
