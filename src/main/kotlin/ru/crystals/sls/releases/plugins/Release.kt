package ru.crystals.sls.releases.plugins

import java.util.Objects

data class Release(val name: String, val localizedName: String, val version: Version, val url: String) {

    fun asCsvRow(): String = "$name, $localizedName, $version, $url"

    fun asHtmlRow(): String = """
        <tr>
          <td>$name</td><td>$localizedName</td><td><a href='$url'>$version</a></td>
        </tr>
    """.trimIndent()

    companion object : Comparator<Release> {

        override fun compare(l: Release?, r: Release?): Int = when {
                Objects.equals(l, r) -> 0
                l == null -> 1
                r == null -> -1
                else -> l.version.compareTo(r.version)
        }

    }

}