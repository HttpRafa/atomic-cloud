# JVM API Integration

The Atomic Cloud API is available through the **GitHub Packages Maven repository**. Authentication may be required to access it. For more details, visit the [package page](https://github.com/HttpRafa/atomic-cloud/packages/2219240).

## Adding the API to Your Project

### **Maven**
To include the API in a **Maven** project, add the following dependency to your `pom.xml` file:

```xml
<dependency>
    <groupId>io.atomic.cloud</groupId>
    <artifactId>api</artifactId>
    <version>0.7.0-SNAPSHOT</version>
</dependency>
```

### **Gradle**
For **Gradle** projects, add the following dependency to your `build.gradle.kts` file:

```kotlin
repositories {
    mavenCentral()
    maven {
        url = uri("https://maven.pkg.github.com/HttpRafa/atomic-cloud")
        name = "GitHub Packages"
        credentials {
            username = project.findProperty("gpr.username") as String?
            password = project.findProperty("gpr.password") as String?
        }
    }
}

dependencies {
    implementation("io.atomic.cloud:api:0.7.0-SNAPSHOT")
}
```

If you're using Groovy-based Gradle (`build.gradle`), use:

```gradle
dependencies {
    implementation 'io.atomic.cloud:api:0.7.0-SNAPSHOT'
}
```
