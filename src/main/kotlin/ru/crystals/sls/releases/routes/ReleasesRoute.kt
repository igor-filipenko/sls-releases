package ru.crystals.sls.releases.routes

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import ru.crystals.sls.releases.client.github.GitHubClient
import ru.crystals.sls.releases.client.github.Converter
import ru.crystals.sls.releases.model.release.Release
import ru.crystals.sls.releases.model.release.Version
import java.util.stream.Collectors

fun Route.releasesRoute(client: GitHubClient, parser: Converter) {
    get("/sls/releases") {
        val useReleaseCandidates = this.context.parameters.get("rc").toBoolean()

        val byVersionType = { r: Release -> if (useReleaseCandidates) true else r.version is Version.Release }

        val result = client.getReleases(parser).stream()
            .filter(byVersionType)
            .collect(Collectors.groupingBy(Release::name))
            .values.stream()
            .map { it.maxWith(Release.Companion) }
            .sorted(Comparator.comparing(Release::name))

        if (context.request.accept()!!.contains("html")) {
            val baseUrl = this.context.request.uri
            val text = result
                .toList()
                .joinToString(
                    prefix = "<table rules=\"all\">",
                    postfix = "</table>",
                    separator = "\n",
                    transform = { release -> release.asHtmlRow(baseUrl, useReleaseCandidates) }
                )
            call.respondText(text, ContentType.Text.Html)
        } else {
            val text = result
                .map(Release::asCsvRow)
                .collect(Collectors.joining("\n")) + "\n"
            call.respondText(text, ContentType.Text.Plain)
        }
    }
    get("/sls/releases/{module}") {
        val useReleaseCandidates = this.context.parameters.get("rc").toBoolean()
        val moduleName = this.context.parameters.get("module")
        val byVersionType = { r: Release -> if (useReleaseCandidates) true else r.version is Version.Release }
        val byModuleName = { r: Release -> r.name == moduleName }

        val result = client.getReleases(parser).stream()
            .filter(byVersionType)
            .filter(byModuleName)
            .sorted(Comparator.comparing(Release::version, Comparator.reverseOrder()))

        if (context.request.accept()!!.contains("html")) {
            val baseUrl = this.context.request.uri
            val text = result
                .toList()
                .joinToString(
                    prefix = "<table rules=\"all\">",
                    postfix = "</table>",
                    separator = "\n",
                    transform = { release -> release.asHtmlRow(baseUrl, useReleaseCandidates) }
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