package dev.hive.rustyhandlebars.actions

import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.platform.lsp.api.LspServerManager
import dev.hive.rustyhandlebars.lsp.RustyHandlebarsLspSupportProvider

class RestartLanguageServerAction : AnAction() {
    override fun actionPerformed(event: AnActionEvent) {
        val project = event.project ?: return
        LspServerManager.getInstance(project).stopAndRestartIfNeeded(
            RustyHandlebarsLspSupportProvider::class.java,
        )
        notify(project, "Rusty Handlebars language server restarted.", NotificationType.INFORMATION)
    }

    override fun update(event: AnActionEvent) {
        event.presentation.isEnabled = event.project != null
    }
}
