# Paper Installation Script
$PROJECT = "paper"
$MINECRAFT_VERSION = "latest"
$BUILD_NUMBER = "latest"
$SERVER_JARFILE = "server.jar"

if ($env:DL_PATH) {
    Write-Host "Using supplied download URL: $($env:DL_PATH)"
    $DOWNLOAD_URL = $env:DL_PATH -replace "{{", '$(' -replace "}}", ')'
} else {
    # Fetch the versions and check for the given version
    $VERSIONS = (Invoke-RestMethod -Uri "https://api.papermc.io/v2/projects/$PROJECT").versions
    $VER_EXISTS = $VERSIONS -contains $MINECRAFT_VERSION

    $LATEST_VERSION = $VERSIONS[-1]

    if ($VER_EXISTS) {
        Write-Host "Version is valid. Using version $MINECRAFT_VERSION"
    } else {
        Write-Host "Specified version not found. Defaulting to the latest $PROJECT version"
        $MINECRAFT_VERSION = $LATEST_VERSION
    }

    # Fetch the builds and check for the given build
    $BUILDS = (Invoke-RestMethod -Uri "https://api.papermc.io/v2/projects/$PROJECT/versions/$MINECRAFT_VERSION").builds
    $BUILD_EXISTS = $BUILDS -contains $BUILD_NUMBER

    $LATEST_BUILD = $BUILDS[-1]

    if ($BUILD_EXISTS) {
        Write-Host "Build is valid for version $MINECRAFT_VERSION. Using build $BUILD_NUMBER"
    } else {
        Write-Host "Using the latest $PROJECT build for version $MINECRAFT_VERSION"
        $BUILD_NUMBER = $LATEST_BUILD
    }

    $JAR_NAME = "$PROJECT-$MINECRAFT_VERSION-$BUILD_NUMBER.jar"

    Write-Host "Version being downloaded"
    Write-Host "MC Version: $MINECRAFT_VERSION"
    Write-Host "Build: $BUILD_NUMBER"
    Write-Host "JAR Name of Build: $JAR_NAME"
    $DOWNLOAD_URL = "https://api.papermc.io/v2/projects/$PROJECT/versions/$MINECRAFT_VERSION/builds/$BUILD_NUMBER/downloads/$JAR_NAME"
}

Write-Host "Running Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $SERVER_JARFILE"

Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $SERVER_JARFILE

Write-Host "Installing required client plugin..."
Invoke-WebRequest -Uri "https://github.com/HttpRafa/atomic-cloud/releases/latest/download/paper-client.jar" -OutFile "paper-client.jar"
New-Item -ItemType Directory -Path "plugins" -Force | Out-Null
Move-Item -Path "paper-client.jar" -Destination "plugins\"
Write-Host "Installed required plugin"

Write-Host "Preparing server..."
Add-Content -Path "eula.txt" -Value "eula=true"
Add-Content -Path "server.properties" -Value "accepts-transfers=true"
Add-Content -Path "bukkit.yml" -Value "settings:"
Add-Content -Path "bukkit.yml" -Value "  connection-throttle: -1"
Write-Host "Ready to start!"