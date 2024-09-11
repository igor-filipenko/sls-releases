package ru.crystals.sls.releases.plugins

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import ru.crystals.sls.releases.client.GitHubClient
import ru.crystals.sls.releases.client.Converter
import ru.crystals.sls.releases.model.Release
import ru.crystals.sls.releases.model.Version
import java.util.stream.Collectors

fun Application.configureRouting(client: GitHubClient, parser: Converter) {
    routing {
        get("/sls/releases") {
            val useReleaseCandidates = this.context.parameters.get("rc").toBoolean()
            println("Using release candidates: $useReleaseCandidates")

            fun byVersionType(r: Release): Boolean =
                if (useReleaseCandidates) true else r.version is Version.Release

            val result = client.getReleases(parser).stream()
                .filter { r -> byVersionType(r) }
                .collect(Collectors.groupingBy { r -> r.name })
                .values.stream()
                .map { list -> list.maxWith(Release.Companion) }
                .sorted(Comparator.comparing { r -> r.name })

            if (context.request.accept()!!.contains("html")) {
                val text = result
                    .toList()
                    .joinToString(
                        prefix = "<table rules=\"all\">",
                        postfix = "</table>",
                        separator = "\n",
                        transform = Release::asHtmlRow
                    )
                call.respondText(text, ContentType.Text.Html)
            } else {
                val text = result
                    .map(Release::asCsvRow)
                    .collect(Collectors.joining("\n")) + "\n"
                call.respondText(text, ContentType.Text.Plain)
            }
        }
    }

}
