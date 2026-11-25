package dev.zlauncher.core

import android.content.Context
import android.content.SharedPreferences
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import dev.zlauncher.core.nativecore.SecretStore
import java.security.GeneralSecurityException
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** The Android key store provider. */
private const val PROVIDER = "AndroidKeyStore"

/** Alias of the key that encrypts the stored values. */
private const val KEY_ALIAS = "zl.secrets"

/** Cipher of the values: AES in GCM mode, with the IV the key store makes for each encryption. */
private const val TRANSFORMATION = "AES/GCM/NoPadding"

/** Length of the GCM tag, in bits. */
private const val TAG_BITS = 128

/** Size of the AES key, in bits. */
private const val KEY_BITS = 256

/** Preferences file of the encrypted values (never in the backup: see the manifest). */
private const val PREFS = "zl_secrets"

/**
 * The core's [SecretStore] on Android (the requirement: API keys in the Keystore). Each value is encrypted with an AES
 * key that is made inside the Android key store and never leaves it; the preferences only hold IV and ciphertext. A
 * value that cannot be decrypted (the key was lost, for example after a restore on another device) reads as absent.
 */
class KeystoreSecrets(context: Context) : SecretStore {
    /** Where the encrypted values are kept. */
    private val prefs: SharedPreferences = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    /** The value of [name], decrypted, or `null`. */
    override fun get(name: String): String? {
        val parts = prefs.getString(name, null)?.split(':', limit = 2)?.takeIf { it.size == 2 }
        return parts?.let { decrypt(it[0], it[1]) }
    }

    /** The text that [data] encrypts with [iv] (both Base64), or `null` when it cannot be read. */
    private fun decrypt(iv: String, data: String): String? = try {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(TAG_BITS, decode(iv)))
        String(cipher.doFinal(decode(data)), Charsets.UTF_8)
    } catch (_: GeneralSecurityException) {
        null
    } catch (_: IllegalArgumentException) {
        null
    }

    /** Encrypts and stores [value] under [name], or removes it; `false` when the key store fails. */
    override fun set(name: String, value: String?): Boolean {
        if (value == null) return prefs.edit().remove(name).commit()
        return try {
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.ENCRYPT_MODE, key())
            val data = cipher.doFinal(value.toByteArray(Charsets.UTF_8))
            prefs.edit().putString(name, "${encode(cipher.iv)}:${encode(data)}").commit()
        } catch (_: GeneralSecurityException) {
            false
        }
    }

    /** The key of the store, made the first time. */
    private fun key(): SecretKey {
        val store = KeyStore.getInstance(PROVIDER).apply { load(null) }
        (store.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }
        val spec = KeyGenParameterSpec.Builder(KEY_ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(KEY_BITS)
            .build()
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, PROVIDER).apply { init(spec) }.generateKey()
    }

    /** Base64 without line breaks. */
    private fun encode(bytes: ByteArray): String = Base64.encodeToString(bytes, Base64.NO_WRAP)

    /** The bytes of [text]. */
    private fun decode(text: String): ByteArray = Base64.decode(text, Base64.NO_WRAP)
}
