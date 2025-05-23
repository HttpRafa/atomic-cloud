# File path and environment variable
$PropertiesFile = "server.properties"
$Port = $env:SERVER_PORT

# Check if the file exists
if (Test-Path $PropertiesFile) {
    # Read the file content
    $FileContent = Get-Content $PropertiesFile

    # Check if the server-port line exists
    if ($FileContent -match "^server-port=") {
        # Update the line
        $UpdatedContent = $FileContent -replace "^server-port=.*", "server-port=$Port"
        Set-Content $PropertiesFile -Value $UpdatedContent
    } else {
        # Append the server-port line
        Add-Content $PropertiesFile -Value "server-port=$Port"
    }
} else {
    # Create the file and add the server-port line
    Set-Content $PropertiesFile -Value "server-port=$Port"
}

.\wrapper.exe -- java -Xms128M -Xmx"$env:SERVER_MEMORY"M -jar $env:SERVER_JARFILE nogui