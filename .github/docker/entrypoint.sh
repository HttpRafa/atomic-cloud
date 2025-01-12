#!/bin/sh

# Check if the environment variable PTERODACTYL is set to true
if [ "$PTERODACTYL" = "true" ]; then
    # Define the URL and the destination folder
    WASM_URL="https://github.com/HttpRafa/atomic-cloud/releases/latest/download/pterodactyl-driver.wasm"
    DEST_FOLDER="drivers/wasm"
    WASM_FILE="$DEST_FOLDER/pterodactyl.wasm"

    # Create the destination folder if it does not exist
    if [ ! -d "$DEST_FOLDER" ]; then
        echo "Creating directory $DEST_FOLDER..."
        mkdir -p "$DEST_FOLDER"
    fi

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
fi

# Run the main command
exec ./controller "$@"