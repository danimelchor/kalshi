#!/usr/bin/env bash

BINARY_NAME="kalshi-bot"   # Replace with your binary name
BINARY_PATH="./target/debug/$BINARY_NAME"
LOG_FILE="/tmp/$BINARY_NAME.log"
DATE=$(date +"%Y-%m-%d")

if cargo check; then
    echo "cargo check succeeded, building release..."
    cargo build --release || { echo "Build failed"; exit 1; }

    # Stop existing binary if running
    if pgrep -x "$BINARY_NAME" > /dev/null; then
        echo "Stopping existing binary..."
        pkill -x "$BINARY_NAME"
    fi

    # Run new binary
    echo "Starting new binary..."
    unbuffer "$BINARY_PATH" system --date "$DATE" >> "$LOG_FILE" 2>&1 &

    echo "Tailing logs..."
    tail -f "$LOG_FILE"
else
    echo "cargo check failed, not restarting."
fi
