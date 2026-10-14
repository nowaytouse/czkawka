// Root Gradle build - declares AGP and its built-in Kotlin runtime.
// The actual app configuration is in app/build.gradle.kts.
buildscript {
    dependencies {
        // AGP's bundled KGP 2.2.10 uses APIs deprecated by Gradle 9.8.
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:2.5.0-Beta1")
    }
}

plugins {
    id("com.android.application") version "9.5.0-alpha09" apply false
}
