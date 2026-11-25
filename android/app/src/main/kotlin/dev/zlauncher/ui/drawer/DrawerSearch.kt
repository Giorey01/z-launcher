package dev.zlauncher.ui.drawer

import java.text.Normalizer

/** Combining marks left after decomposing a letter. */
private val COMBINING_MARKS = Regex("\\p{M}+")

/** [text] without accents and in lower case, so `Pokémon` and `pokemon` compare equal. */
internal fun foldForSearch(text: String): String =
    Normalizer.normalize(text, Normalizer.Form.NFD).replace(COMBINING_MARKS, "").lowercase()

/**
 * Whether an app labelled [label] matches the search [query]: accents and case do not matter and blanks
 * around the query are ignored. An empty query matches everything.
 */
internal fun matchesSearch(label: String, query: String): Boolean {
    val needle = foldForSearch(query.trim())
    return needle.isEmpty() || foldForSearch(label).contains(needle)
}

/**
 * Position in the drawer grid of the item with focus key [focused]: the [recentKeys] come first, then, when
 * there are any, one divider cell, then the [appKeys]. `null` when the key is not shown (for instance a
 * recent app while the search hides the recent row).
 */
internal fun gridIndexOf(focused: String, recentKeys: List<String>, appKeys: List<String>): Int? {
    val inRecent = recentKeys.indexOf(focused)
    if (inRecent >= 0) return inRecent
    val inApps = appKeys.indexOf(focused)
    if (inApps < 0) return null
    return inApps + if (recentKeys.isEmpty()) 0 else recentKeys.size + 1
}
