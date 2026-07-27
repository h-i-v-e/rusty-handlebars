package dev.hive.rustyhandlebars.settings

import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.openapi.components.StoragePathMacros
import com.intellij.openapi.components.service
import com.intellij.openapi.project.Project

@State(
    name = "RustyHandlebarsSettings",
    storages = [Storage(StoragePathMacros.WORKSPACE_FILE)],
)
class RustyHandlebarsSettings :
    PersistentStateComponent<RustyHandlebarsSettings.State> {

    data class State(
        var serverPath: String = "",
        var legacyFileGlobs: MutableList<String> = mutableListOf(),
    )

    private var state = State()

    override fun getState() = state
    override fun loadState(state: State) {
        this.state = state
    }

    var serverPath: String
        get() = state.serverPath
        set(value) {
            state.serverPath = value.trim()
        }

    var legacyFileGlobs: List<String>
        get() = state.legacyFileGlobs
        set(value) {
            state.legacyFileGlobs = value.map(String::trim)
                .filter(String::isNotEmpty)
                .distinct()
                .toMutableList()
        }

    companion object {
        fun getInstance(project: Project): RustyHandlebarsSettings = project.service()
    }
}
