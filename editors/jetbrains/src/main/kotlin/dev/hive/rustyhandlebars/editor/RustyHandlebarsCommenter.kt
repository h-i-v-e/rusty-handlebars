package dev.hive.rustyhandlebars.editor

import com.intellij.lang.Commenter

class RustyHandlebarsCommenter : Commenter {
    override fun getLineCommentPrefix(): String? = null
    override fun getBlockCommentPrefix() = "{{!--"
    override fun getBlockCommentSuffix() = "--}}"
    override fun getCommentedBlockCommentPrefix(): String? = null
    override fun getCommentedBlockCommentSuffix(): String? = null
}
