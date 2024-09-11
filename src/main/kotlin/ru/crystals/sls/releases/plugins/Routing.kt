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
    Pair("communications", "Коммуникации"),
    Pair("coupons", "Купоны"),
    Pair("customers", "Покупатели"),
    Pair("discounts", "Скидки"),
    Pair("dwh", "Аналитика"),
    Pair("gateway", "gateway"),
    Pair("limits", "Лимиты"),
    Pair("offers", "Офферы"),
    Pair("purchases", "Чеки"),
    Pair("registrations", "Регистрации"),
    Pair("segments", "Сегменты"),
    Pair("triggers", "Триггеры"),
    Pair("scheduler", "Scheduler"),
    Pair("superset", "Superset"),
    Pair("superset-integration", "superset-integration"),
)

fun Application.configureRouting(client: GitHubClient) {
    routing {
        get("/sls/releases") {
            val text = client.getReleases(Parser(knownModules)).stream()
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
