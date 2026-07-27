package dev.hive.rustyhandlebars.lsp

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VfsUtilCore
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor
import com.intellij.platform.lsp.api.LspServer
import com.intellij.platform.lsp.api.lsWidget.LspServerWidgetItem
import dev.hive.rustyhandlebars.RustyHandlebarsIcons
import dev.hive.rustyhandlebars.RustyHandlebarsFileType
import dev.hive.rustyhandlebars.server.ServerBinaryManager
import dev.hive.rustyhandlebars.settings.LegacyFileMatcher
import dev.hive.rustyhandlebars.settings.RustyHandlebarsSettings

class RustyHandlebarsLspSupportProvider : LspServerSupportProvider {
    override fun fileOpened(
        project: Project,
        file: VirtualFile,
        serverStarter: LspServerSupportProvider.LspServerStarter,
    ) {
        if (RustyHandlebarsLspServerDescriptor.isSupported(project, file)) {
            serverStarter.ensureServerStarted(
                RustyHandlebarsLspServerDescriptor(project),
            )
        }
    }

    override fun createLspServerWidgetItem(
        lspServer: LspServer,
        currentFile: VirtualFile?,
    ) = LspServerWidgetItem(
        lspServer,
        currentFile,
        RustyHandlebarsIcons.File,
        dev.hive.rustyhandlebars.settings.RustyHandlebarsConfigurable::class.java,
    )
}

class RustyHandlebarsLspServerDescriptor(project: Project) :
    ProjectWideLspServerDescriptor(project, "Rusty Handlebars") {

    override fun isSupportedFile(file: VirtualFile): Boolean =
        isSupported(project, file)

    override fun createCommandLine(): GeneralCommandLine {
        val executable = ServerBinaryManager(project).executable()
        return GeneralCommandLine(executable.toString()).apply {
            project.basePath?.let(::withWorkDirectory)
        }
    }

    override val lsp4jServerClass: Class<out org.eclipse.lsp4j.services.LanguageServer>
        get() = RustyHandlebarsLanguageServer::class.java

    companion object {
        fun isSupported(project: Project, file: VirtualFile): Boolean {
            if (file.fileType == RustyHandlebarsFileType.INSTANCE ||
                file.extension.equals("rhbs", ignoreCase = true)
            ) {
                return true
            }
            val basePath = project.basePath ?: return false
            val base = com.intellij.openapi.vfs.LocalFileSystem.getInstance()
                .findFileByPath(basePath) ?: return false
            val relative = VfsUtilCore.getRelativePath(file, base, '/') ?: return false
            return RustyHandlebarsSettings.getInstance(project).legacyFileGlobs.any {
                LegacyFileMatcher.matches(it, relative)
            }
        }
    }
}
