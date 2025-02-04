package ru.crystals.sls.releases.routes

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import org.unbrokendome.base62.Base62
import ru.crystals.sls.releases.model.transaction.Transaction
import java.time.LocalDateTime
import java.time.OffsetDateTime

fun Route.transactionsRoute() {
    val zoneOffset = OffsetDateTime.now().getOffset()
    get("/sls/transactions/{id}") {
        val id = call.parameters["id"]!!
        try {
            val part = Base62.decodeArray(id)
            val internalId = part[0]
            val seconds = part[1]
            val created = LocalDateTime.ofEpochSecond(seconds, 0, zoneOffset)
            call.respond(Transaction(internalId, created))
        } catch (e: Exception) {
            println("Failed to decode transaction ID: ${e.stackTraceToString()}")
            call.respond(HttpStatusCode.BadRequest, "Invalid transaction ID: '$id'")
        }
    }
}