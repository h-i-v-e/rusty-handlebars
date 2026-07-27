package dev.hive.rustyhandlebars.actions

import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import dev.hive.rustyhandlebars.lsp.RustyHandlebarsLspServers

class ReloadProjectIndexAction : AnAction() {
    override fun actionPerformed(event: AnActionEvent) {
        val project = event.project ?: return
        val reloaded = RustyHandlebarsLspServers.reloadProject(project)
        notify(
            project,
            if (reloaded) {
                "Rusty Handlebars project index reloaded."
            } else {
                "The project index could not be reloaded. Check the IDE log for details."
            },
            if (reloaded) NotificationType.INFORMATION else NotificationType.WARNING,
        )
    }

    override fun update(event: AnActionEvent) {
        event.presentation.isEnabled =
            event.project?.let { RustyHandlebarsLspServers.running(it) != null } == true
    }
}
