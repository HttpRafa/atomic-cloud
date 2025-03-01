#!/usr/bin/env bash

# Check if the server jar already exists
if [ -f "${SERVER_JARFILE}" ]; then
    echo "The ${SERVER_JARFILE} already exists. Skipping script execution."
    exit 0
fi

if [ -n "${DL_PATH}" ]; then
    echo -e "Using supplied download url: ${DL_PATH}"
    DOWNLOAD_URL=$(eval echo $(echo ${DL_PATH} | sed -e 's/{{/${/g' -e 's/}}/}/g'))
else
    # Fetch the Minecraft version manifest from Mojang
    MANIFEST=$(curl -sSL https://launchermeta.mojang.com/mc/game/version_manifest.json)

    # Extract the latest release and snapshot versions
    LATEST_RELEASE=$(echo "$MANIFEST" | grep -oP '"release":\s*"\K[^"]+')
    LATEST_SNAPSHOT=$(echo "$MANIFEST" | grep -oP '"snapshot":\s*"\K[^"]+')

    # Determine which version to use:
    # If VERSION is not set or is "latest", use the latest release.
    # If VERSION is "snapshot", use the latest snapshot.
    # Otherwise, check if the specified VERSION exists in the manifest; if not, default to the latest release.
    if [ -z "${VERSION}" ] || [ "${VERSION}" == "latest" ]; then
        SELECTED_VERSION=${LATEST_RELEASE}
    elif [ "${VERSION}" == "snapshot" ]; then
        SELECTED_VERSION=${LATEST_SNAPSHOT}
    else
        if echo "$MANIFEST" | grep -q -E "\"id\": *\"${VERSION}\""; then
            SELECTED_VERSION=${VERSION}
        else
            echo -e "Specified version not found. Defaulting to the latest release version."
            SELECTED_VERSION=${LATEST_RELEASE}
        fi
    fi

    # Extract the URL for the selected version’s manifest.
    # Remove newlines, then break the "versions" array objects onto separate lines.
    VERSION_URL=$(echo "$MANIFEST" | tr -d '\n' | sed -E 's/\}, *\{/\}\n\{/g' | grep -E "\"id\": *\"${SELECTED_VERSION}\"" | grep -oP '"url":\s*"\K[^"]+')

    # Fetch the selected version's manifest
    VERSION_MANIFEST=$(curl -sSL "${VERSION_URL}")

    # Extract the vanilla server download URL from the version manifest.
    # Using a non-greedy match so that we correctly find the "url" key inside the "server" object.
    DOWNLOAD_URL=$(echo "$VERSION_MANIFEST" | tr -d '\n' | grep -oP '"server":\s*{.*?"url":\s*"\K[^"]+')

    echo "Version being downloaded:"
    echo -e "MC Version: ${SELECTED_VERSION}"
fi

echo "Running curl -o ${SERVER_JARFILE} ${DOWNLOAD_URL}"

curl -o ${SERVER_JARFILE} ${DOWNLOAD_URL}

echo "Installing wrapper binary..."
curl -o wrapper -L https://github.com/HttpRafa/atomic-cloud/releases/latest/download/wrapper-linux-x86_64
echo "Installed wrapper binary"
echo "Preparing server..."
echo "eula=true" >> eula.txt
echo "accepts-transfers=true" >> server.properties
echo "Ready to start!"
exit 0