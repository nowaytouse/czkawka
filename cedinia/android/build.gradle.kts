// Root Gradle build – only declares the AGP plugin version.
// The actual app configuration is in app/build.gradle.kts.
plugins {
    id("com.android.application") version "9.4.1" apply false
}
