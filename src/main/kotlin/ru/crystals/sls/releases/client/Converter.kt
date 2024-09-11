package ru.crystals.sls.releases.client

import ru.crystals.sls.releases.model.Release
import ru.crystals.sls.releases.model.Version
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
            url = ghr.url
        )
        )
    }

}