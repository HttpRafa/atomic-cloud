#!/bin/sh

# Define the destination folder
DEST_FOLDER="plugins/"

# Create the destination folder if it does not exist
if [ ! -d "$DEST_FOLDER" ]; then
    echo "Creating directory $DEST_FOLDER..."
    mkdir -p "$DEST_FOLDER"
fi

# Function to download the .wasm file
download_wasm() {
    local WASM_URL=$1
    local WASM_FILE=$2

    # Check if the .wasm file already exists
    if [ -f "$WASM_FILE" ]; then
        echo "File $WASM_FILE already exists. Skipping download."
    else
        echo "File $WASM_FILE does not exist. Proceeding with download."

        # Download the .wasm file
        echo "Downloading $WASM_URL..."
        curl -L -o $WASM_FILE $WASM_URL

        echo "Download complete."
    fi
}

# Check if the environment variable PTERODACTYL is set to true
if [ "$PTERODACTYL" = "true" ]; then
    WASM_URL="https://github.com/HttpRafa/atomic-cloud/releases/latest/download/pelican.wasm"
    WASM_FILE="$DEST_FOLDER/pelican.wasm"
    download_wasm $WASM_URL $WASM_FILE
fi

# Check if the environment variable LOCAL is set to true
if [ "$LOCAL" = "true" ]; then
    WASM_URL="https://github.com/HttpRafa/atomic-cloud/releases/latest/download/local.wasm"
    WASM_FILE="$DEST_FOLDER/local.wasm"
    download_wasm $WASM_URL $WASM_FILE
fi

# Run the main command
exec ./controller "$@"