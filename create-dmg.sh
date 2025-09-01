#!/bin/bash

# 🏴‍☠️ PortKill DMG Creator
# Creates a professional DMG installer for macOS

set -e

echo "🏴‍☠️  Creating PortKill DMG..."
echo "============================="

# Build the app first
echo "🔨 Building PortKill..."
./build-macos.sh

# Create DMG directory structure
DIST_DIR="dist"
DMG_NAME="$DIST_DIR/PortKill-v1.3.dmg"
DMG_TEMP="build/PortKill-temp.dmg"
APP_NAME="PortKill.app"
BUILD_DIR="build"

# Create dist directory
mkdir -p "$DIST_DIR"

echo "📦 Creating app bundle..."

# Create build directory and app bundle structure
mkdir -p "$BUILD_DIR/$APP_NAME/Contents/MacOS"
mkdir -p "$BUILD_DIR/$APP_NAME/Contents/Resources"

# Copy the binary
cp "./target/release/port-kill" "$BUILD_DIR/$APP_NAME/Contents/MacOS/port-kill-binary"

# Copy assets directory to Resources
cp -R "assets" "$BUILD_DIR/$APP_NAME/Contents/Resources/assets"

# Create launch script
cat > "$BUILD_DIR/$APP_NAME/Contents/MacOS/PortKill" << 'LAUNCH_EOF'
#!/bin/bash

# PortKill Launch Script
# Default ports to monitor
DEFAULT_PORTS="3000,8000,8080,3001,5000,5173"

# Get the directory of this script
SCRIPT_DIR="$(dirname "$0")"

# Launch PortKill with default settings
exec "$SCRIPT_DIR/port-kill-binary" --ports "$DEFAULT_PORTS" "$@"
LAUNCH_EOF

chmod +x "$BUILD_DIR/$APP_NAME/Contents/MacOS/PortKill"
chmod +x "$BUILD_DIR/$APP_NAME/Contents/MacOS/port-kill-binary"

# Create Info.plist
cat > "$BUILD_DIR/$APP_NAME/Contents/Info.plist" << 'PLIST_EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleExecutable</key>
	<string>PortKill</string>
	<key>CFBundleIdentifier</key>
	<string>com.martinhessmann.portkill</string>
	<key>CFBundleName</key>
	<string>PortKill</string>
	<key>CFBundleDisplayName</key>
	<string>PortKill</string>
	<key>CFBundleVersion</key>
	<string>1.3</string>
	<key>CFBundleShortVersionString</key>
	<string>1.3</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleSignature</key>
	<string>????</string>
	<key>LSMinimumSystemVersion</key>
	<string>10.15</string>
	<key>LSUIElement</key>
	<true/>
	<key>NSHighResolutionCapable</key>
	<true/>
	<key>CFBundleIconFile</key>
	<string>PortKill</string>
	<key>NSAppleEventsUsageDescription</key>
	<string>PortKill needs to monitor system processes to identify development servers running on specified ports.</string>
	<key>LSApplicationCategoryType</key>
	<string>public.app-category.developer-tools</string>
</dict>
</plist>
PLIST_EOF

# Create icon set
echo "🎨 Creating app icon..."
mkdir -p "$BUILD_DIR/PortKill.iconset"
magick assets/large.png -resize 16x16 "$BUILD_DIR/PortKill.iconset/icon_16x16.png"
magick assets/large.png -resize 32x32 "$BUILD_DIR/PortKill.iconset/icon_16x16@2x.png"
magick assets/large.png -resize 32x32 "$BUILD_DIR/PortKill.iconset/icon_32x32.png"
magick assets/large.png -resize 64x64 "$BUILD_DIR/PortKill.iconset/icon_32x32@2x.png"
magick assets/large.png -resize 128x128 "$BUILD_DIR/PortKill.iconset/icon_128x128.png"
magick assets/large.png -resize 256x256 "$BUILD_DIR/PortKill.iconset/icon_128x128@2x.png"
magick assets/large.png -resize 256x256 "$BUILD_DIR/PortKill.iconset/icon_256x256.png"
magick assets/large.png -resize 512x512 "$BUILD_DIR/PortKill.iconset/icon_256x256@2x.png"
magick assets/large.png -resize 512x512 "$BUILD_DIR/PortKill.iconset/icon_512x512.png"
magick assets/large.png -resize 1024x1024 "$BUILD_DIR/PortKill.iconset/icon_512x512@2x.png"

iconutil -c icns "$BUILD_DIR/PortKill.iconset" -o "$BUILD_DIR/$APP_NAME/Contents/Resources/PortKill.icns"

# Create DMG
echo "📦 Creating DMG..."

# Calculate size needed (app size + overhead)
APP_SIZE=$(du -sm "$BUILD_DIR/$APP_NAME" | cut -f1)
DMG_SIZE=$((APP_SIZE + 50)) # Add 50MB overhead

# Create temporary DMG
hdiutil create -size "${DMG_SIZE}m" -fs HFS+ -volname "PortKill" -attach "$DMG_TEMP"

# Copy app to DMG
cp -R "$BUILD_DIR/$APP_NAME" "/Volumes/PortKill/"

# Create Applications folder link
ln -s /Applications "/Volumes/PortKill/Applications"

# Style the DMG window
echo "📋 Styling DMG window..."

# Create AppleScript to style the DMG window
osascript << 'APPLESCRIPT_EOF'
tell application "Finder"
    tell disk "PortKill"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set the bounds of container window to {400, 100, 920, 420}
        set viewOptions to the icon view options of container window
        set arrangement of viewOptions to not arranged
        set icon size of viewOptions to 72
        set position of item "PortKill.app" of container window to {140, 120}
        set position of item "Applications" of container window to {380, 120}
        close
        open
        update without registering applications
        delay 2
    end tell
end tell
APPLESCRIPT_EOF

# Unmount and convert to compressed DMG
hdiutil detach "/Volumes/PortKill"
hdiutil convert "$DMG_TEMP" -format UDZO -o "$DMG_NAME"

# Clean up build artifacts
rm -rf "$BUILD_DIR"

echo ""
echo "✅ DMG Created Successfully!"
echo "============================"
echo "📁 DMG File: $DMG_NAME"
echo "📏 Size: $(du -h "$DMG_NAME" | cut -f1)"
echo ""
echo "🎯 Installation Instructions:"
echo "1. Double-click the DMG file"
echo "2. Drag 'PortKill' to Applications folder"
echo "3. Launch from Applications or Spotlight"
echo ""
echo "✨ Features included:"
echo "  • Custom poison bottle icons"
echo "  • Color-coded status system"
echo "  • Fixed macOS stability issues"
echo "  • Enhanced tray menu"
echo ""
echo "Happy coding! 🚀"
