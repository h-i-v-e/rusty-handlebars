package dev.hive.rustyhandlebars.editor

import com.intellij.lang.BracePair
import com.intellij.lang.PairedBraceMatcher
import com.intellij.psi.PsiFile
import com.intellij.psi.tree.IElementType

class RustyHandlebarsBraceMatcher : PairedBraceMatcher {
    override fun getPairs() = arrayOf(
        BracePair(RustyHandlebarsTokens.OPEN, RustyHandlebarsTokens.CLOSE, false),
        BracePair(RustyHandlebarsTokens.OPEN_RAW, RustyHandlebarsTokens.CLOSE_RAW, false),
    )

    override fun isPairedBracesAllowedBeforeType(
        lbraceType: IElementType,
        contextType: IElementType?,
    ) = true

    override fun getCodeConstructStart(file: PsiFile?, openingBraceOffset: Int) =
        openingBraceOffset
}
