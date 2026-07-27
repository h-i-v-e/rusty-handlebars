import org.jetbrains.intellij.platform.gradle.IntelliJPlatformType
import org.jetbrains.intellij.platform.gradle.TestFrameworkType
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    kotlin("jvm") version "2.2.20"
    id("org.jetbrains.intellij.platform")
}

dependencies {
    testImplementation(kotlin("test"))
    testImplementation("junit:junit:4.13.2")

    intellijPlatform {
        rustRover("2025.3.1")
        testFramework(TestFrameworkType.Platform)
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_21
        freeCompilerArgs.add("-Xjvm-default=all")
    }
}

intellijPlatform {
    instrumentCode = false

    pluginConfiguration {
        name = "Rusty Handlebars"
        version = project.version.toString()
        ideaVersion {
            sinceBuild = "253.29346"
            untilBuild = provider { null }
        }
        vendor {
            name = "h-i-v-e"
            email = "nosnhoj.emorej@gmail.com"
            url = "https://github.com/h-i-v-e/rusty-handlebars"
        }
    }

    pluginVerification {
        ides {
            create(IntelliJPlatformType.RustRover, "2025.3.1")
            create(IntelliJPlatformType.RustRover, "2025.3.6")
            create(IntelliJPlatformType.RustRover, "2026.1.4")
            create(IntelliJPlatformType.RustRover, "2026.2")
        }
    }

    signing {
        certificateChain = providers.environmentVariable("CERTIFICATE_CHAIN")
        privateKey = providers.environmentVariable("PRIVATE_KEY")
        password = providers.environmentVariable("PRIVATE_KEY_PASSWORD")
    }

    publishing {
        token = providers.environmentVariable("PUBLISH_TOKEN")
    }
}

tasks {
    test {
        useJUnit()
    }

    buildSearchableOptions {
        enabled = false
    }
}
