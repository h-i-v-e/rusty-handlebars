package dev.hive.rustyhandlebars

import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

class RustyHandlebarsFileType private constructor() :
    LanguageFileType(RustyHandlebarsLanguage) {

    override fun getName() = "Rusty Handlebars"
    override fun getDescription() = "Rusty Handlebars template"
    override fun getDefaultExtension() = "rhbs"
    override fun getIcon(): Icon = RustyHandlebarsIcons.File

    companion object {
        @JvmField
        val INSTANCE = RustyHandlebarsFileType()
    }
}
