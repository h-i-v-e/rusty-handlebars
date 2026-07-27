package dev.hive.rustyhandlebars.editor

import com.intellij.lang.Language
import com.intellij.lang.html.HTMLLanguage
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.FileViewProvider
import com.intellij.psi.FileViewProviderFactory
import com.intellij.psi.MultiplePsiFilesPerDocumentFileViewProvider
import com.intellij.psi.PsiManager
import com.intellij.psi.tree.IElementType
import com.intellij.webcore.template.AbstractTemplateLanguageFileViewProvider
import dev.hive.rustyhandlebars.RustyHandlebarsLanguage

class RustyHandlebarsFileViewProvider(
    manager: PsiManager,
    file: VirtualFile,
    physical: Boolean,
    templateDataLanguage: Language = HTMLLanguage.INSTANCE,
) : AbstractTemplateLanguageFileViewProvider(
    manager,
    file,
    physical,
    templateDataLanguage,
) {
    override fun getBaseLanguage(): Language = RustyHandlebarsLanguage

    override fun getTemplateDataType(): IElementType =
        RustyHandlebarsParserDefinition.TEMPLATE_DATA

    override fun cloneInner(
        fileCopy: VirtualFile,
    ): MultiplePsiFilesPerDocumentFileViewProvider =
        RustyHandlebarsFileViewProvider(
            manager,
            fileCopy,
            false,
            templateDataLanguage,
        )
}

class RustyHandlebarsFileViewProviderFactory : FileViewProviderFactory {
    override fun createFileViewProvider(
        file: VirtualFile,
        language: Language,
        manager: PsiManager,
        eventSystemEnabled: Boolean,
    ): FileViewProvider =
        RustyHandlebarsFileViewProvider(
            manager,
            file,
            eventSystemEnabled,
        )
}
