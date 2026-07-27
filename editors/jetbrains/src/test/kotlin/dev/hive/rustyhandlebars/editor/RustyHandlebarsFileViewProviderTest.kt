package dev.hive.rustyhandlebars.editor

import com.intellij.lang.html.HTMLLanguage
import com.intellij.psi.FileTypeFileViewProviders
import com.intellij.psi.PsiManager
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.intellij.testFramework.LightVirtualFile
import dev.hive.rustyhandlebars.RustyHandlebarsFileType
import dev.hive.rustyhandlebars.RustyHandlebarsLanguage

class RustyHandlebarsFileViewProviderTest : BasePlatformTestCase() {
    fun testProvidesTemplateAndHtmlPsiTrees() {
        val virtualFile = LightVirtualFile(
            "template.rhbs",
            RustyHandlebarsFileType.INSTANCE,
            "<p>{{name}}</p>",
        )
        val factory = FileTypeFileViewProviders.INSTANCE.forFileType(
            RustyHandlebarsFileType.INSTANCE,
        )
        assertInstanceOf(factory, RustyHandlebarsFileViewProviderFactory::class.java)
        val provider = factory.createFileViewProvider(
            virtualFile,
            RustyHandlebarsLanguage,
            PsiManager.getInstance(project),
            false,
        )

        assertContainsElements(
            provider.languages,
            RustyHandlebarsLanguage,
            HTMLLanguage.INSTANCE,
        )
        assertNotNull(provider.getPsi(RustyHandlebarsLanguage))
        assertNotNull(provider.getPsi(HTMLLanguage.INSTANCE))
    }
}
