package dev.hive.rustyhandlebars.lsp

import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.ProjectActivity
import com.intellij.openapi.vfs.VirtualFileManager
import com.intellij.openapi.vfs.newvfs.BulkFileListener
import com.intellij.openapi.vfs.newvfs.events.VFileEvent
import com.intellij.openapi.util.io.FileUtil
import com.intellij.util.Alarm

class ProjectIndexWatcher : ProjectActivity {
    override suspend fun execute(project: Project) {
        val basePath = project.basePath
            ?.let(FileUtil::toSystemIndependentName)
            ?.trimEnd('/')
            ?: return
        val alarm = Alarm(Alarm.ThreadToUse.POOLED_THREAD, project)
        project.messageBus.connect(project).subscribe(
            VirtualFileManager.VFS_CHANGES,
            object : BulkFileListener {
                override fun after(events: List<VFileEvent>) {
                    if (events.any { it.isProjectIndexInput(basePath) }) {
                        alarm.cancelAllRequests()
                        alarm.addRequest(
                            { RustyHandlebarsLspServers.reloadProject(project) },
                            500,
                        )
                    }
                }
            },
        )
    }

    private fun VFileEvent.isProjectIndexInput(basePath: String): Boolean {
        val normalized = FileUtil.toSystemIndependentName(path)
        if (normalized != basePath && !normalized.startsWith("$basePath/")) return false
        val relative = normalized.removePrefix(basePath).trimStart('/')
        if (relative.startsWith("target/") || relative.startsWith(".git/")) return false
        val name = normalized.substringAfterLast('/')
        return name == "Cargo.toml" || name == "Cargo.lock" || name.endsWith(".rs")
    }
}
