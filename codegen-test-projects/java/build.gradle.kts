plugins {
    java
}

repositories {
    // Use Maven Central for resolving dependencies.
    mavenCentral()
}

dependencies {

    implementation("com.amazon.ion:ion-java:1.11.4")

    // Use JUnit Jupiter for testing.
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
    testImplementation("org.junit.jupiter:junit-jupiter:5.7.1")
}

val ionSchemaSourceCodeDir = "${layout.projectDirectory}/src/main/ion-schema"
val generatedIonSchemaModelDir = "${layout.buildDirectory.get()}/generated/ion-schema/java"
sourceSets {
    main {
        java.srcDir(generatedIonSchemaModelDir)
    }
}


tasks {
    val ionCodegen = create<Exec>("ionCodegen") {
        inputs.files(ionSchemaSourceCodeDir)
        outputs.file(generatedIonSchemaModelDir)

        val ionCli = System.getenv("ION_CLI") ?: "ion"

        commandLine(ionCli)
            .args(
                "beta", "generate",
                "-l", "java",
                "-s", "struct_with_fields.isl",
                "-d", ionSchemaSourceCodeDir,
                "-o", generatedIonSchemaModelDir,
            )
            .workingDir(rootProject.projectDir)
    }

    withType<JavaCompile> {
        options.encoding = "UTF-8"
        // The `release` option is not available for the Java 8 compiler, but if we're building with Java 8 we don't
        // need it anyway.
        if (JavaVersion.current() != JavaVersion.VERSION_1_8) {
            options.release.set(8)
        }

        dependsOn(ionCodegen)
    }
}

tasks.named<Test>("test") {
    // Use JUnit Platform for unit tests.
    useJUnitPlatform()
}

