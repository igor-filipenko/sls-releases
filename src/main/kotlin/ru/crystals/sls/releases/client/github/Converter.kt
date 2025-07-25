package ru.crystals.sls.releases.client.github

import ru.crystals.sls.releases.model.release.Release
import ru.crystals.sls.releases.model.release.Version
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale
import java.util.function.Consumer
import java.util.regex.Pattern

class Converter(private val knownModules: Map<String, String>) {

    internal fun convert(ghr: GitHubRelease, consumer: Consumer<Release>) {
        val pattern = Pattern.compile("^(.*)-v(\\d+).(\\d+).(\\d+)(-RC\\d+)?\$")
        val matcher = pattern.matcher(ghr.tagName)
        if (!matcher.matches())
            return

        val moduleName = matcher.group(1)
        if (!knownModules.containsKey(moduleName))
            return;

        val version =
            if (matcher.group(5) != null) {
                Version.Candidate(
                    major = matcher.group(2).toInt(),
                    minor = matcher.group(3).toInt(),
                    patch = matcher.group(4).toInt(),
                    number = matcher.group(5).substringAfter("-RC").toInt()
                )
            } else {
                Version.Release(
                    major = matcher.group(2).toInt(),
                    minor = matcher.group(3).toInt(),
                    patch = matcher.group(4).toInt()
                )
            }

        consumer.accept(
            Release(
                name = moduleName,
                localizedName = knownModules.getOrDefault(moduleName, moduleName),
                version = version,
                url = ghr.url,
                dateTime = convert(ghr.publishTime)
            )
        )
    }

    internal fun convert(publishTime: String): String {
        return try {
            val instant = Instant.parse(publishTime)

            val formatter = DateTimeFormatter.ofPattern("MMM d, yyyy 'at' h:mm a")
                .withLocale(Locale.getDefault())
                .withZone(ZoneId.systemDefault())

            formatter.format(instant)
        } catch (e: Exception) {
            println("Failed to parse date: $publishTime, $e")
            publishTime
        }
    }
}