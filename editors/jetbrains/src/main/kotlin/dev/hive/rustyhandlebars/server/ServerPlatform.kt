package dev.hive.rustyhandlebars.server

enum class ServerPlatform(
    val resourceDirectory: String,
    val executableName: String,
) {
    DARWIN_ARM64("darwin-arm64", "rusty-handlebars-language-server"),
    DARWIN_X64("darwin-x64", "rusty-handlebars-language-server"),
    LINUX_ARM64("linux-arm64", "rusty-handlebars-language-server"),
    LINUX_X64("linux-x64", "rusty-handlebars-language-server"),
    WINDOWS_X64("win32-x64", "rusty-handlebars-language-server.exe");

    companion object {
        fun detect(
            osName: String = System.getProperty("os.name"),
            architecture: String = System.getProperty("os.arch"),
        ): ServerPlatform? {
            val os = osName.lowercase()
            val arch = architecture.lowercase()
            return when {
                (os.contains("mac") || os.contains("darwin")) &&
                    arch in ARM64_NAMES -> DARWIN_ARM64
                (os.contains("mac") || os.contains("darwin")) &&
                    arch in X64_NAMES -> DARWIN_X64
                os.contains("linux") && arch in ARM64_NAMES -> LINUX_ARM64
                os.contains("linux") && arch in X64_NAMES -> LINUX_X64
                os.contains("windows") && arch in X64_NAMES -> WINDOWS_X64
                else -> null
            }
        }

        private val ARM64_NAMES = setOf("aarch64", "arm64")
        private val X64_NAMES = setOf("x86_64", "amd64", "x64")
    }
}
