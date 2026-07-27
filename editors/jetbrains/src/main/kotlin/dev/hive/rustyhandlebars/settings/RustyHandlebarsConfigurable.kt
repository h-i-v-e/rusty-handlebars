package dev.hive.rustyhandlebars.settings

import com.intellij.openapi.options.Configurable
import com.intellij.openapi.options.ConfigurationException
import com.intellij.openapi.project.Project
import com.intellij.platform.lsp.api.LspServerManager
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.components.JBTextArea
import com.intellij.ui.components.JBTextField
import com.intellij.util.ui.FormBuilder
import java.nio.file.Files
import java.nio.file.InvalidPathException
import java.nio.file.Path
import javax.swing.JComponent
import javax.swing.JPanel

class RustyHandlebarsConfigurable(private val project: Project) : Configurable {
    private var panel: JPanel? = null
    private val serverPath = JBTextField()
    private val legacyGlobs = JBTextArea(6, 40)

    override fun getDisplayName() = "Rusty Handlebars"

    override fun createComponent(): JComponent {
        legacyGlobs.emptyText.text = "templates/**/*.hbs"
        panel = FormBuilder.createFormBuilder()
            .addLabeledComponent(
                JBLabel("Language server path:"),
                serverPath,
                true,
            )
            .addTooltip("Leave empty to use the server bundled with the plugin.")
            .addLabeledComponent(
                JBLabel("Legacy template globs (one per line):"),
                JBScrollPane(legacyGlobs),
                true,
            )
            .addComponentFillVertically(JPanel(), 0)
            .panel
        reset()
        return panel!!
    }

    override fun isModified(): Boolean {
        val settings = RustyHandlebarsSettings.getInstance(project)
        return serverPath.text.trim() != settings.serverPath ||
            parsedGlobs() != settings.legacyFileGlobs
    }

    override fun apply() {
        val settings = RustyHandlebarsSettings.getInstance(project)
        val previousPath = settings.serverPath
        val previousGlobs = settings.legacyFileGlobs
        val configuredPath = serverPath.text.trim()
        if (configuredPath.isNotEmpty()) {
            val path = try {
                Path.of(configuredPath)
            } catch (_: InvalidPathException) {
                throw ConfigurationException("The language server path is invalid.")
            }
            if (!Files.isRegularFile(path)) {
                throw ConfigurationException(
                    "The language server path must name an existing file.",
                )
            }
        }
        parsedGlobs().forEach {
            try {
                LegacyFileMatcher.compile(it)
            } catch (error: IllegalArgumentException) {
                throw ConfigurationException(error.message)
            }
        }
        settings.apply {
            serverPath = configuredPath
            legacyFileGlobs = parsedGlobs()
        }
        if (previousPath != settings.serverPath || previousGlobs != settings.legacyFileGlobs) {
            LspServerManager.getInstance(project).stopAndRestartIfNeeded(
                dev.hive.rustyhandlebars.lsp.RustyHandlebarsLspSupportProvider::class.java,
            )
        }
    }

    override fun reset() {
        val settings = RustyHandlebarsSettings.getInstance(project)
        serverPath.text = settings.serverPath
        legacyGlobs.text = settings.legacyFileGlobs.joinToString("\n")
    }

    override fun disposeUIResources() {
        panel = null
    }

    private fun parsedGlobs() = legacyGlobs.text.lineSequence()
        .map(String::trim)
        .filter(String::isNotEmpty)
        .distinct()
        .toList()
}
