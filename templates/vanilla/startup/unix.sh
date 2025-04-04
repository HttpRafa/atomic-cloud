#!/usr/bin/env bash

# File path and environment variable
PROPERTIES_FILE="server.properties"
PORT="${SERVER_PORT}"

# Update or add the server-port line
if [ -f "$PROPERTIES_FILE" ]; then
  # Check if the line exists and update it; otherwise, append it
  if grep -q "^server-port=" "$PROPERTIES_FILE"; then
    sed -i "s/^server-port=.*/server-port=$PORT/" "$PROPERTIES_FILE"
  else
    echo "server-port=$PORT" >> "$PROPERTIES_FILE"
  fi
else
  # Create the file and add the line
  echo "server-port=$PORT" > "$PROPERTIES_FILE"
fi

# Set permissions and execute the server
chmod +x wrapper
exec ./wrapper -- java -Xms128M  -Xmx${SERVER_MEMORY}M -Dterminal.jline=false -Dterminal.ansi=true -jar $SERVER_JARFILE nogui