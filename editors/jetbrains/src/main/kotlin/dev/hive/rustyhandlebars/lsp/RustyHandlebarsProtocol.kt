package dev.hive.rustyhandlebars.lsp

import org.eclipse.lsp4j.TextDocumentIdentifier
import org.eclipse.lsp4j.jsonrpc.services.JsonRequest
import org.eclipse.lsp4j.services.LanguageServer
import java.util.concurrent.CompletableFuture

interface RustyHandlebarsLanguageServer : LanguageServer {
    @JsonRequest("rustyHandlebars/showGeneratedRust")
    fun showGeneratedRust(
        document: TextDocumentIdentifier,
    ): CompletableFuture<String>

    @JsonRequest("rustyHandlebars/reloadProject")
    fun reloadProject(): CompletableFuture<Boolean>
}
