package dev.zlauncher

import dev.zlauncher.pods.work.PodWorkBook
import dev.zlauncher.pods.work.WorkSummary
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** The book of the Pods' running tasks: keys, progress, what the notification shows. */
class PodWorkBookTest {
    @Test
    fun oneTaskPerKey() {
        val book = PodWorkBook()
        assertTrue(book.add(PodWorkBook.keyOf("a", "dl"), "Download"))
        assertFalse("same id of the same Pod", book.add(PodWorkBook.keyOf("a", "dl"), "Again"))
        assertTrue("same id of another Pod", book.add(PodWorkBook.keyOf("b", "dl"), "Other"))
        assertEquals(setOf("a:dl", "b:dl"), book.running.value)
    }

    @Test
    fun summaryIsTheFirstTaskAndTheCount() {
        val book = PodWorkBook()
        assertNull(book.summary())
        book.add("a:1", "First")
        book.add("a:2", "Second")
        book.progress("a:1", "12 MB of 100 MB", 140)
        book.progress("a:9", "not running", 5)
        assertEquals(WorkSummary("First", "12 MB of 100 MB", 100, 2), book.summary())
        book.remove("a:1")
        assertEquals(WorkSummary("Second", null, null, 1), book.summary())
        book.remove("a:2")
        assertNull(book.summary())
        assertEquals(emptySet<String>(), book.running.value)
    }

    @Test
    fun runningOfOnePodWithoutItsPrefix() {
        val book = PodWorkBook()
        assertEquals(setOf("dl", "x:y"), book.runningOf("a", setOf("a:dl", "a:x:y", "ab:dl", "b:dl")))
    }
}
