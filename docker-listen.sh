#!/bin/bash

sudo apt-get install inotify-tools

# Find all directories matching */src (one level deep), excluding web-client/src
WATCH_DIRS=$(find . -type d -name "src" | grep -v "web-client/src")

# Initial build and bringing up services
echo "Initial build and bringing up services..."
docker compose down -t 0
docker compose build
docker compose up -d

# Start following logs in the background
docker compose logs -f &
LOGS_PID=$!

# Start watching for file changes in the found src directories
echo "Watching for changes in the following directories:"
echo "$WATCH_DIRS"

# Debounce mechanism variables
DEBOUNCE_TIMEOUT=2   # Time in seconds to wait after detecting changes
LAST_CHANGE_TIME=0
TRIGGER_BUILD=false

# Function to handle the rebuilding process
rebuild() {
    echo "Rebuilding containers and re-upping services..."
    
    docker compose down -t 0
    docker compose build
    docker compose up -d

    # Kill the previous logs process
    kill $LOGS_PID

    # Start following logs again
    docker compose logs -f &
    LOGS_PID=$!
}

# Run inotifywait on all src directories and watch for changes (modify, create, delete, move)
inotifywait -m -r -e modify,create,delete,move $WATCH_DIRS |
while read -r path action file; do
    # Skip if the path includes "web-client/src"
    if [[ "$path" == *"web-client/src"* ]]; then
        echo "Change detected in web-client/src, skipping rebuild..."
        continue
    fi

    echo "Change detected in $path$file ($action)"

    CURRENT_TIME=$(date +%s)
    TIME_DIFF=$((CURRENT_TIME - LAST_CHANGE_TIME))

    if [[ $TIME_DIFF -lt $DEBOUNCE_TIMEOUT ]]; then
        # If changes occurred within the debounce timeout, skip rebuilding immediately
        echo "Skipping immediate rebuild due to recent changes."
    else
        # Set the trigger to true and store the last change time
        TRIGGER_BUILD=true
        LAST_CHANGE_TIME=$CURRENT_TIME
    fi

    # Start a background process to trigger rebuild after the debounce timeout
    if [[ $TRIGGER_BUILD == true ]]; then
        TRIGGER_BUILD=false
        sleep $DEBOUNCE_TIMEOUT
        rebuild
    fi
done
