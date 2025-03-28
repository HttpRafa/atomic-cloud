plugins {
    java

    id("com.diffplug.spotless")
}

tasks {
    processResources {
        dependsOn(spotlessApply)
    }
}

spotless {
    java {
        palantirJavaFormat()
        removeUnusedImports()
        trimTrailingWhitespace()
        endWithNewline()
    }
}