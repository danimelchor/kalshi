#!/usr/bin/env bash

BINARY_NAME="kalshi-bot"

# Stop existing binary if running
if pgrep -x "$BINARY_NAME" > /dev/null; then
    echo "Stopping existing binary..."
    pkill -x "$BINARY_NAME"
fi
