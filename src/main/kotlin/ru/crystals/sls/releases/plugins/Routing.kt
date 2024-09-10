package ru.crystals.sls.releases.plugins

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import ru.crystals.sls.releases.client.GitHubClient
import ru.crystals.sls.releases.client.GitHubRelease
import java.util.function.Consumer
import java.util.stream.Collectors

val knownModules = hashMapOf(
    Pair("accumulations", "Накопления"),
    Pair("bonuses", "Бонусы"),
    Pair("coupons", "Купоны"),
    Pair("customers", "Покупатели"),
)

fun Application.configureRouting(client: GitHubClient) {
    routing {
        get("/sls/releases") {
            val releases = client.getReleases()

            val text = releases.stream()
                .mapMulti(Parser(knownModules)::parse)
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

fun toRelease(ghr: GitHubRelease, consumer: Consumer<Release>) {

}
