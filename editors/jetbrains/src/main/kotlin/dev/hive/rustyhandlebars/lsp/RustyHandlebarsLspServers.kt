package dev.hive.rustyhandlebars.lsp

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServer
import com.intellij.platform.lsp.api.LspServerManager
import com.intellij.platform.lsp.api.LspServerState

object RustyHandlebarsLspServers {
    fun running(project: Project, file: VirtualFile? = null): LspServer? =
        LspServerManager.getInstance(project)
            .getServersForProvider(RustyHandlebarsLspSupportProvider::class.java)
            .firstOrNull {
                it.state == LspServerState.Running &&
                    (file == null || it.descriptor.isSupportedFile(file))
            }

    fun reloadProject(project: Project): Boolean {
        val server = running(project) ?: return false
        return server.sendRequestSync(10_000) {
            (it as RustyHandlebarsLanguageServer).reloadProject()
        } ?: false
    }
}
