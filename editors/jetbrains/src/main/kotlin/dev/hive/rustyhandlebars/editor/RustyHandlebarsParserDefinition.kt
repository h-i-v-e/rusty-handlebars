package dev.hive.rustyhandlebars.editor

import com.intellij.extapi.psi.PsiFileBase
import com.intellij.lang.ASTNode
import com.intellij.lang.ParserDefinition
import com.intellij.lang.PsiBuilder
import com.intellij.lang.PsiParser
import com.intellij.lexer.Lexer
import com.intellij.openapi.fileTypes.FileType
import com.intellij.openapi.project.Project
import com.intellij.psi.FileViewProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.tree.IFileElementType
import com.intellij.psi.tree.OuterLanguageElementType
import com.intellij.psi.tree.TokenSet
import com.intellij.psi.templateLanguages.TemplateDataElementType
import dev.hive.rustyhandlebars.RustyHandlebarsFileType
import dev.hive.rustyhandlebars.RustyHandlebarsLanguage

class RustyHandlebarsParserDefinition : ParserDefinition {
    override fun createLexer(project: Project?): Lexer = RustyHandlebarsLexer()

    override fun createParser(project: Project?): PsiParser =
        PsiParser { root, builder -> parseAll(root, builder) }

    override fun getFileNodeType() = FILE
    override fun getWhitespaceTokens() = TokenSet.WHITE_SPACE
    override fun getCommentTokens() = RustyHandlebarsTokens.COMMENTS
    override fun getStringLiteralElements() = RustyHandlebarsTokens.STRINGS

    override fun createElement(node: ASTNode): PsiElement =
        com.intellij.extapi.psi.ASTWrapperPsiElement(node)

    override fun createFile(viewProvider: FileViewProvider): PsiFile =
        RustyHandlebarsFile(viewProvider)

    private fun parseAll(root: com.intellij.psi.tree.IElementType, builder: PsiBuilder): ASTNode {
        val marker = builder.mark()
        while (!builder.eof()) builder.advanceLexer()
        marker.done(root)
        return builder.treeBuilt
    }

    companion object {
        @JvmField
        val FILE = IFileElementType(RustyHandlebarsLanguage)

        @JvmField
        val OUTER = OuterLanguageElementType(
            "RUSTY_HANDLEBARS_OUTER",
            RustyHandlebarsLanguage,
        )

        @JvmField
        val TEMPLATE_DATA = TemplateDataElementType(
            "RUSTY_HANDLEBARS_TEMPLATE_DATA",
            RustyHandlebarsLanguage,
            RustyHandlebarsTokens.DATA,
            OUTER,
        )
    }
}

private class RustyHandlebarsFile(viewProvider: FileViewProvider) :
    PsiFileBase(viewProvider, RustyHandlebarsLanguage) {

    override fun getFileType(): FileType = RustyHandlebarsFileType.INSTANCE
    override fun toString() = "Rusty Handlebars template"
}
