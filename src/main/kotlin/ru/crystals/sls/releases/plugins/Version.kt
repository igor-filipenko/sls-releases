package ru.crystals.sls.releases.plugins

import java.util.*
import java.util.stream.Stream

sealed interface Version : Comparable<Version> {
    val major: Int
    val minor: Int
    val patch: Int

    data class Release(override val major: Int, override val minor: Int, override val patch: Int) : Version {

        override fun toString(): String {
            return """
                $major.$minor.$patch
            """.trimIndent()
        }

    }

    data class Candidate(override val major: Int, override val minor: Int, override val patch: Int, val number: Int) : Version {

        override fun toString(): String {
            return """
                $major.$minor.$patch-RC$number
            """.trimIndent()
        }

    }

    override fun compareTo(other: Version): Int {
        val thisNumber = if (this is Candidate) this.number else Int.MAX_VALUE
        val otherNumber = if (other is Candidate) other.number else Int.MAX_VALUE

        val results = Stream.of(
            this.major.compareTo(other.major),
            this.minor.compareTo(other.minor),
            this.patch.compareTo(other.patch),
            thisNumber.compareTo(otherNumber)
        )

        return results.takeWhile { r -> r != 0 }.findFirst().orElse(0)
    }

}