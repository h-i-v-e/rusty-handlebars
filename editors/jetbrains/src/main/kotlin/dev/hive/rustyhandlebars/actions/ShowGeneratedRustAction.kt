package dev.hive.rustyhandlebars.actions

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileTypes.FileTypeManager
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.testFramework.LightVirtualFile
import dev.hive.rustyhandlebars.lsp.RustyHandlebarsLanguageServer
import dev.hive.rustyhandlebars.lsp.RustyHandlebarsLspServerDescriptor
import dev.hive.rustyhandlebars.lsp.RustyHandlebarsLspServers

class ShowGeneratedRustAction : AnAction() {
    override fun update(event: AnActionEvent) {
        val project = event.project
        val file = event.getData(CommonDataKeys.VIRTUAL_FILE)
        event.presentation.isEnabledAndVisible =
            project != null && file != null &&
                RustyHandlebarsLspServerDescriptor.isSupported(project, file)
    }

    override fun actionPerformed(event: AnActionEvent) {
        val project = event.project ?: return
        val file = event.getData(CommonDataKeys.VIRTUAL_FILE) ?: return
        val server = RustyHandlebarsLspServers.running(project, file)
        if (server == null) {
            notify(
                project,
                "The Rusty Handlebars language server is not running.",
                NotificationType.WARNING,
            )
            return
        }

        ProgressManager.getInstance().run(
            object : Task.Backgroundable(project, "Generating Rust", false) {
                override fun run(indicator: ProgressIndicator) {
                    try {
                        val source = server.sendRequestSync(10_000) {
                            (it as RustyHandlebarsLanguageServer)
                                .showGeneratedRust(server.getDocumentIdentifier(file))
                        } ?: error("The language server returned no generated source.")
                        ApplicationManager.getApplication().invokeLater {
                            val rustFileType = FileTypeManager.getInstance()
                                .getFileTypeByExtension("rs")
                            val generated = LightVirtualFile(
                                "${file.nameWithoutExtension}.generated.rs",
                                rustFileType,
                                source,
                            )
                            generated.isWritable = false
                            FileEditorManager.getInstance(project)
                                .openFile(generated, true)
                        }
                    } catch (error: Exception) {
                        notify(
                            project,
                            "Unable to generate Rust: ${error.message ?: error.javaClass.simpleName}",
                            NotificationType.ERROR,
                        )
                    }
                }
            },
        )
    }
}

internal fun notify(
    project: com.intellij.openapi.project.Project,
    content: String,
    type: NotificationType,
) {
    NotificationGroupManager.getInstance()
        .getNotificationGroup("Rusty Handlebars")
        .createNotification(content, type)
        .notify(project)
}
