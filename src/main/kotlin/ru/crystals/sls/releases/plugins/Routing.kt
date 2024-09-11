package ru.crystals.sls.releases.plugins

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import ru.crystals.sls.releases.client.GitHubClient
import java.util.stream.Collectors

fun Application.configureRouting(client: GitHubClient, parser: Parser) {
    routing {
        get("/sls/releases") {
            val useReleaseCandidates = this.context.parameters.get("rc").toBoolean()

            fun byVersionType(r: Release): Boolean =
                if (useReleaseCandidates) true else r.version is Version.Release

            val text = client.getReleases(parser).stream()
                .filter { r -> byVersionType(r) }
                .collect(Collectors.groupingBy { r -> r.name })
                .values.stream()
                .map { list -> list.maxWith(Release.Companion) }
                .sorted(Comparator.comparing { r -> r.name })
                .map(Release::asCsvRow)
                .collect(Collectors.joining("\n")) + "\n"

            call.respondText(text, ContentType.Text.Plain)
        }
    }
}
