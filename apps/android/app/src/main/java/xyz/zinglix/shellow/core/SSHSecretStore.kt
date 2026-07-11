package xyz.zinglix.shellow.core

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import androidx.core.content.edit
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

enum class SSHSecretKind(val storageName: String) {
  Password("password"),
  PrivateKey("private-key"),
  Passphrase("passphrase"),
}

class SSHSecretStore(context: Context) {
  private val appContext = context.applicationContext
  private val preferences =
    appContext.getSharedPreferences("shellow.ssh.secrets", Context.MODE_PRIVATE)

  fun hasSecret(
    profile: HostProfile,
    kind: SSHSecretKind,
  ): Boolean = loadSecret(profile, kind) != null

  fun hasKeySecret(
    keyId: String,
    kind: SSHSecretKind,
  ): Boolean = loadKeySecret(keyId, kind) != null

  fun loadSecret(
    profile: HostProfile,
    kind: SSHSecretKind,
  ): String? = loadAccountSecret(account(profile, kind))

  fun loadKeySecret(
    keyId: String,
    kind: SSHSecretKind,
  ): String? = loadAccountSecret(keyAccount(keyId, kind))

  private fun loadAccountSecret(
    account: String,
  ): String? {
    val encoded = preferences.getString(account, null) ?: return null
    val parts = encoded.split(":")
    if (parts.size != 2) return null

    return runCatching {
      val iv = Base64.decode(parts[0], Base64.NO_WRAP)
      val ciphertext = Base64.decode(parts[1], Base64.NO_WRAP)
      val cipher = Cipher.getInstance(Transformation)
      cipher.init(Cipher.DECRYPT_MODE, getOrCreateKey(), GCMParameterSpec(GcmTagBits, iv))
      String(cipher.doFinal(ciphertext), Charsets.UTF_8).takeIf { it.isNotEmpty() }
    }.getOrNull()
  }

  fun saveSecret(
    secret: String,
    profile: HostProfile,
    kind: SSHSecretKind,
  ) {
    saveAccountSecret(secret, account(profile, kind))
  }

  fun saveKeySecret(
    secret: String,
    keyId: String,
    kind: SSHSecretKind,
  ) {
    saveAccountSecret(secret, keyAccount(keyId, kind))
  }

  private fun saveAccountSecret(
    secret: String,
    account: String,
  ) {
    if (secret.isEmpty()) return

    val cipher = Cipher.getInstance(Transformation)
    cipher.init(Cipher.ENCRYPT_MODE, getOrCreateKey())
    val ciphertext = cipher.doFinal(secret.toByteArray(Charsets.UTF_8))
    val encoded =
      listOf(
        Base64.encodeToString(cipher.iv, Base64.NO_WRAP),
        Base64.encodeToString(ciphertext, Base64.NO_WRAP),
      ).joinToString(":")

    preferences.edit { putString(account, encoded) }
  }

  fun deleteSecret(
    profile: HostProfile,
    kind: SSHSecretKind,
  ) {
    preferences.edit { remove(account(profile, kind)) }
  }

  fun deleteKeySecret(
    keyId: String,
    kind: SSHSecretKind,
  ) {
    preferences.edit { remove(keyAccount(keyId, kind)) }
  }

  private fun account(
    profile: HostProfile,
    kind: SSHSecretKind,
  ): String = "${profile.id}.${kind.storageName}"

  private fun keyAccount(
    keyId: String,
    kind: SSHSecretKind,
  ): String = "key.$keyId.${kind.storageName}"

  private fun getOrCreateKey(): SecretKey {
    val keyStore = KeyStore.getInstance(AndroidKeyStore).apply { load(null) }
    val existing = keyStore.getEntry(KeyAlias, null) as? KeyStore.SecretKeyEntry
    if (existing != null) {
      return existing.secretKey
    }

    val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, AndroidKeyStore)
    val spec =
      KeyGenParameterSpec
        .Builder(KeyAlias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
        .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
        .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
        .setRandomizedEncryptionRequired(true)
        .build()
    generator.init(spec)
    return generator.generateKey()
  }

  private companion object {
    const val AndroidKeyStore = "AndroidKeyStore"
    const val KeyAlias = "shellow_ssh_secrets_v1"
    const val Transformation = "AES/GCM/NoPadding"
    const val GcmTagBits = 128
  }
}
