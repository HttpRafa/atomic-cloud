plugins {
    id("maven-publish")
}

publishing {
    publications {
        create<MavenPublication>("mavenCommon") {
            from(components["java"])
        }
    }

    repositories {
        maven {
            url = uri("https://repo.external.rafa.run/snapshots")
            name = "rafaRepository"
            credentials(PasswordCredentials::class)
        }
    }
}