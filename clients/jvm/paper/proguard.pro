# Specify the library jars to be included in the ProGuard processing
-libraryjars <java.home>/jmods/java.base.jmod(!**.jar;!module-info.class)

# Keep all classes in the io.atomic.cloud.paper package
-keep class io.atomic.cloud.paper.** { *; }

# Keep all classes in the io.atomic.cloud.api package
-keep class io.atomic.cloud.api.** { *; }

# Keep all classes in the io.atomic.cloud.dependencies.grpc.netty.shaded package
-keep class io.atomic.cloud.dependencies.grpc.netty.shaded.** { *; }

# Do not warn about any missing classes
-dontwarn **

# Remove unused classes
-dontobfuscate
-dontoptimize