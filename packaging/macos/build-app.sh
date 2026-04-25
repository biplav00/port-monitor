#!/usr/bin/env bash
set -euo pipefail

BIN_PATH="target/release/port-monitor"
APP_PATH="target/release/PortMonitor.app"

rm -rf "$APP_PATH"
mkdir -p "$APP_PATH/Contents/MacOS"
mkdir -p "$APP_PATH/Contents/Resources"

cp "$BIN_PATH" "$APP_PATH/Contents/MacOS/port-monitor"
cp packaging/macos/Info.plist "$APP_PATH/Contents/Info.plist"
cp assets/icon.png "$APP_PATH/Contents/Resources/icon.png"
chmod +x "$APP_PATH/Contents/MacOS/port-monitor"
echo "Built $APP_PATH"
