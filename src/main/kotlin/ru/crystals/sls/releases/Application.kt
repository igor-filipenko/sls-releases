package ru.crystals.sls.releases

import com.fasterxml.jackson.core.util.DefaultIndenter
import com.fasterxml.jackson.core.util.DefaultPrettyPrinter
import com.fasterxml.jackson.databind.SerializationFeature
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule
import io.ktor.serialization.jackson.*
import io.ktor.server.application.*
import io.ktor.server.plugins.contentnegotiation.*
import io.ktor.server.routing.*
import ru.crystals.sls.releases.client.github.Converter
import ru.crystals.sls.releases.client.github.GitHubClient
import ru.crystals.sls.releases.routes.releasesRoute
import ru.crystals.sls.releases.routes.transactionsRoute

fun main(args: Array<String>) {
    io.ktor.server.netty.EngineMain.main(args)
}

fun Application.module() {
    install(ContentNegotiation) {
        jackson {
            configure(SerializationFeature.INDENT_OUTPUT, true)
            configure(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS, false)
            setDefaultPrettyPrinter(DefaultPrettyPrinter().apply {
                indentArraysWith(DefaultPrettyPrinter.FixedSpaceIndenter.instance)
                indentObjectsWith(DefaultIndenter("  ", "\n"))
            })
            registerModule(JavaTimeModule())  // support java.time.* types
        }
    }

    val token = environment.config.property("github.token").getString()
    val knownModules= environment.config.config("sls.modules").toMap()
        .mapValues { it.value.toString() }

    routing {
        releasesRoute(GitHubClient(token), Converter(knownModules))
        transactionsRoute()
    }
}
