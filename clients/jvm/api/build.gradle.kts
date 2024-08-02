plugins {
    id("maven-publish")
}

publishing {
    publications {
        register<MavenPublication>("gpr") {
            from(components["java"])
        }
    }

    repositories {
        maven {
            url = uri("https://maven.pkg.github.com/HttpRafa/atomic-cloud")
            name = "GitHubPackages"
            credentials {
                username = System.getenv("GITHUB_ACTOR")
                password = System.getenv("GITHUB_TOKEN")
            }
        }
    }
}