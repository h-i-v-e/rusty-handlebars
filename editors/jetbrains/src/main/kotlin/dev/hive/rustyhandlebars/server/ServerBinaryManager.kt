package dev.hive.rustyhandlebars.server

import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.application.PathManager
import com.intellij.openapi.extensions.PluginId
import com.intellij.openapi.project.Project
import dev.hive.rustyhandlebars.settings.RustyHandlebarsSettings
import java.io.IOException
import java.nio.file.AtomicMoveNotSupportedException
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.nio.file.attribute.PosixFilePermission
import java.security.MessageDigest

class ServerBinaryManager(private val project: Project) {
    fun executable(): Path {
        val custom = RustyHandlebarsSettings.getInstance(project).serverPath
        if (custom.isNotEmpty()) return validateCustom(Path.of(custom))

        val platform = ServerPlatform.detect()
            ?: throw ServerBinaryException(
                "No bundled Rusty Handlebars language server is available for " +
                    "${System.getProperty("os.name")} ${System.getProperty("os.arch")}.",
            )
        return extract(platform)
    }

    private fun validateCustom(path: Path): Path {
        if (!Files.isRegularFile(path)) {
            throw ServerBinaryException(
                "The configured Rusty Handlebars language server does not exist: $path",
            )
        }
        return path.toAbsolutePath().normalize()
    }

    private fun extract(platform: ServerPlatform): Path {
        val version = PluginManagerCore.getPlugin(PLUGIN_ID)?.version ?: "development"
        val directory = Path.of(
            PathManager.getSystemPath(),
            "rusty-handlebars",
            version,
            platform.resourceDirectory,
        )
        Files.createDirectories(directory)
        val target = directory.resolve(platform.executableName)
        val resourcePath =
            "/server/${platform.resourceDirectory}/${platform.executableName}"
        val checksumPath = "$resourcePath.sha256"
        val expectedChecksum = resource(checksumPath).bufferedReader().use {
            it.readText().trim().substringBefore(' ')
        }
        if (Files.isRegularFile(target) && sha256(target) == expectedChecksum) {
            makeExecutable(target)
            return target
        }

        val temporary = Files.createTempFile(directory, "server-", ".tmp")
        try {
            resource(resourcePath).use { input ->
                Files.copy(input, temporary, StandardCopyOption.REPLACE_EXISTING)
            }
            val actualChecksum = sha256(temporary)
            if (!actualChecksum.equals(expectedChecksum, ignoreCase = true)) {
                throw ServerBinaryException(
                    "The bundled Rusty Handlebars language server failed its checksum.",
                )
            }
            makeExecutable(temporary)
            try {
                Files.move(
                    temporary,
                    target,
                    StandardCopyOption.ATOMIC_MOVE,
                    StandardCopyOption.REPLACE_EXISTING,
                )
            } catch (_: AtomicMoveNotSupportedException) {
                Files.move(temporary, target, StandardCopyOption.REPLACE_EXISTING)
            }
            makeExecutable(target)
            return target
        } finally {
            Files.deleteIfExists(temporary)
        }
    }

    private fun resource(path: String) =
        javaClass.getResourceAsStream(path)
            ?: throw ServerBinaryException(
                "The plugin does not contain the language server for this platform. " +
                    "Set a custom server path in Rusty Handlebars settings.",
            )

    private fun makeExecutable(path: Path) {
        if (ServerPlatform.detect()?.name?.startsWith("WINDOWS") == true) return
        try {
            val permissions = Files.getPosixFilePermissions(path).toMutableSet()
            permissions += PosixFilePermission.OWNER_EXECUTE
            Files.setPosixFilePermissions(path, permissions)
        } catch (_: UnsupportedOperationException) {
            if (!path.toFile().setExecutable(true, true)) {
                throw ServerBinaryException("Unable to make the language server executable: $path")
            }
        }
        if (!Files.isExecutable(path)) {
            throw ServerBinaryException("The language server is not executable: $path")
        }
    }

    private fun sha256(path: Path): String {
        val digest = MessageDigest.getInstance("SHA-256")
        Files.newInputStream(path).use { input ->
            val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
            while (true) {
                val read = input.read(buffer)
                if (read < 0) break
                digest.update(buffer, 0, read)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    companion object {
        private val PLUGIN_ID = PluginId.getId("dev.hive.rusty-handlebars")
    }
}

class ServerBinaryException(message: String, cause: IOException? = null) :
    RuntimeException(message, cause)
