package dev.hive.rustyhandlebars.editor

import com.intellij.codeInsight.editorActions.SimpleTokenSetQuoteHandler
import com.intellij.psi.tree.TokenSet

class RustyHandlebarsQuoteHandler :
    SimpleTokenSetQuoteHandler(TokenSet.create(RustyHandlebarsTokens.STRING))
