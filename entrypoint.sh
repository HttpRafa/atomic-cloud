#!/bin/sh

# Check if the environment variable PTERODACTYL is set to true
if [ "$PTERODACTYL" = "true" ]; then
    # Define the URL and the destination folder
    ZIP_URL="https://nightly.link/HttpRafa/atomic-cloud/workflows/rust_pterodactyl/main/pterodactyl.zip"
    ZIP_FILE="pterodactyl.zip"
    DEST_FOLDER="drivers/wasm/"
    WASM_FILE="$DEST_FOLDER/pterodactyl.wasm"

    # Check if the .wasm file already exists
    if [ -f "$WASM_FILE" ]; then
        echo "File $WASM_FILE already exists. Skipping download."
    else
        echo "File $WASM_FILE does not exist. Proceeding with download."

        # Download the .zip file
        echo "Downloading $ZIP_URL..."
        curl -L -o $ZIP_FILE $ZIP_URL

        # Extract the .zip file
        echo "Extracting $ZIP_FILE to $DEST_FOLDER..."
        unzip $ZIP_FILE -d $DEST_FOLDER

        # Remove the .zip file
        echo "Removing $ZIP_FILE..."
        rm $ZIP_FILE

        echo "Download and extraction complete."
    fi
else
    echo "PTERODACTYL is not set to true. Skipping download."
fi

# Run the main command
./controller "$@"