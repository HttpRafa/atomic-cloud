# Check if the server jar already exists
if (Test-Path $env:SERVER_JARFILE) {
    Write-Host "The $env:SERVER_JARFILE already exists. Skipping script execution."
    exit 0
}

if ($env:DL_PATH) {
    Write-Host "Using supplied download url: $env:DL_PATH"
    $DOWNLOAD_URL = $env:DL_PATH -replace "{{", '$(' -replace "}}", ')'
} else {
    # Fetch the Minecraft version manifest from Mojang
    $MANIFEST = Invoke-RestMethod -Uri "https://launchermeta.mojang.com/mc/game/version_manifest.json"
    $LATEST_RELEASE = $MANIFEST.latest.release
    $LATEST_SNAPSHOT = $MANIFEST.latest.snapshot

    # Determine which version to use:
    # If VERSION is not set or is "latest", use the latest release.
    # If VERSION is "snapshot", use the latest snapshot.
    # Otherwise, if the specified version exists in the manifest use it; if not, default to the latest release.
    if (-not $env:VERSION -or $env:VERSION -eq "latest") {
        $SELECTED_VERSION = $LATEST_RELEASE
    }
    elseif ($env:VERSION -eq "snapshot") {
        $SELECTED_VERSION = $LATEST_SNAPSHOT
    }
    else {
        if ($MANIFEST.versions | Where-Object { $_.id -eq $env:VERSION }) {
            $SELECTED_VERSION = $env:VERSION
        }
        else {
            Write-Host "Specified version not found. Defaulting to the latest release version."
            $SELECTED_VERSION = $LATEST_RELEASE
        }
    }

    # Extract the URL for the selected version's manifest.
    $VERSION_URL = ($MANIFEST.versions | Where-Object { $_.id -eq $SELECTED_VERSION }).url

    # Fetch the selected version's manifest
    $VERSION_MANIFEST = Invoke-RestMethod -Uri $VERSION_URL

    # Extract the vanilla server download URL from the version manifest.
    $DOWNLOAD_URL = $VERSION_MANIFEST.downloads.server.url

    Write-Host "Version being downloaded:"
    Write-Host "MC Version: $SELECTED_VERSION"
}

Write-Host "Running Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $env:SERVER_JARFILE"
Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $env:SERVER_JARFILE

Write-Host "Installing wrapper binary..."
Invoke-WebRequest -Uri "https://github.com/HttpRafa/atomic-cloud/releases/latest/download/wrapper-windows-x86_64.exe" -OutFile "wrapper.exe"
Write-Host "Installed wrapper binary"

Write-Host "Preparing server..."
Add-Content -Path "eula.txt" -Value "eula=true"
Add-Content -Path "server.properties" -Value "accepts-transfers=true"
Write-Host "Ready to start!"
exit 0