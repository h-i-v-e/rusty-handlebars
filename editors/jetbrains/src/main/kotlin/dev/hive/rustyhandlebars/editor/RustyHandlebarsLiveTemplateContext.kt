package dev.hive.rustyhandlebars.editor

import com.intellij.codeInsight.template.TemplateActionContext
import com.intellij.codeInsight.template.TemplateContextType
import dev.hive.rustyhandlebars.RustyHandlebarsLanguage

class RustyHandlebarsLiveTemplateContext :
    TemplateContextType("Rusty Handlebars") {

    override fun isInContext(templateActionContext: TemplateActionContext): Boolean =
        templateActionContext.file.language.isKindOf(RustyHandlebarsLanguage)
}
