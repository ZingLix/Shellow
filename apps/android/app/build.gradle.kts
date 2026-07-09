plugins {
  alias(libs.plugins.android.application)
}

apply(plugin = "org.jetbrains.kotlin.android")
apply(plugin = "org.jetbrains.kotlin.plugin.compose")

fun nonBlankEnv(name: String): String? = System.getenv(name)?.takeIf { it.isNotBlank() }

val ciVersionCode = nonBlankEnv("ANDROID_VERSION_CODE")?.toIntOrNull() ?: 1
val ciVersionName = nonBlankEnv("ANDROID_VERSION_NAME") ?: "0.1.0"
val releaseKeystoreFile = nonBlankEnv("ANDROID_KEYSTORE_FILE")?.let { file(it) }
val releaseKeystorePassword = nonBlankEnv("ANDROID_KEYSTORE_PASSWORD")
val releaseKeyAlias = nonBlankEnv("ANDROID_KEY_ALIAS")
val releaseKeyPassword = nonBlankEnv("ANDROID_KEY_PASSWORD") ?: releaseKeystorePassword
val hasReleaseSigning =
    releaseKeystoreFile != null &&
        releaseKeystorePassword != null &&
        releaseKeyAlias != null &&
        releaseKeyPassword != null

android {
    namespace = "xyz.zinglix.shellow"
    compileSdk = 36
    ndkVersion = "27.1.12297006"

    defaultConfig {
        applicationId = "xyz.zinglix.shellow"
        minSdk = 26
        targetSdk = 36
        versionCode = ciVersionCode
        versionName = ciVersionName

        externalNativeBuild {
            cmake {
                arguments += "-DSHELLOW_ROOT=${rootProject.projectDir.parentFile?.parentFile?.absolutePath}"
            }
        }

        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    signingConfigs {
        if (hasReleaseSigning) {
            create("release") {
                storeFile = releaseKeystoreFile
                storePassword = releaseKeystorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.findByName("release")
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    buildFeatures {
      compose = true
      aidl = false
      buildConfig = false
      shaders = false
    }

    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
            version = "3.22.1"
        }
    }

    packaging {
      jniLibs {
        pickFirsts += listOf("**/libshellow_ffi.so")
      }
      resources {
        excludes += "/META-INF/{AL2.0,LGPL2.1}"
      }
    }
}

extensions.configure<org.jetbrains.kotlin.gradle.dsl.KotlinAndroidProjectExtension>("kotlin") {
    jvmToolchain(17)
}

dependencies {
  // Core Android dependencies
  implementation(libs.androidx.core.ktx)
  implementation(libs.androidx.lifecycle.runtime.ktx)
  implementation(libs.androidx.activity.compose)

  // Arch Components
  implementation(libs.androidx.lifecycle.runtime.compose)
  implementation(libs.androidx.lifecycle.viewmodel.compose)

  // Compose
  implementation(libs.androidx.compose.ui)
  implementation(libs.androidx.compose.ui.tooling.preview)
  implementation(libs.androidx.compose.material3)
  // Tooling
  debugImplementation(libs.androidx.compose.ui.tooling)

  testImplementation(libs.junit)

  androidTestImplementation(libs.androidx.test.core)
  androidTestImplementation(libs.androidx.test.ext.junit)
  androidTestImplementation(libs.androidx.test.runner)
}
