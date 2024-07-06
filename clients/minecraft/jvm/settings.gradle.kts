rootProject.name = "atomic-cloud"

// Include
includeSubProjects("api", "paper")

fun includeSubProjects(vararg names: String) {
    names.forEach { name ->
        include(":$name")
        println("> Module :$name ADDED")
    }
}