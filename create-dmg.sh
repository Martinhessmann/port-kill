#!/bin/bash

# 🏴‍☠️ PortKill Enhanced DMG Creator
# Creates a professional DMG installer for macOS

set -e

echo "🏴‍☠️  Creating PortKill Enhanced DMG..."
echo "======================================"

# Build the app first
echo "🔨 Building PortKill Enhanced..."
./build-macos.sh

# Create DMG directory structure
DMG_NAME="PortKill-Enhanced-v1.3.dmg"
DMG_TEMP="PortKill-Enhanced-temp.dmg"
APP_NAME="PortKill Enhanced.app"

echo "📦 Creating app bundle..."

# Create app bundle directory
mkdir -p "/tmp/$APP_NAME/Contents/MacOS"
mkdir -p "/tmp/$APP_NAME/Contents/Resources"

# Copy the binary
cp "./target/release/port-kill" "/tmp/$APP_NAME/Contents/MacOS/PortKill"

# Create launch script
cat > "/tmp/$APP_NAME/Contents/MacOS/PortKill" << 'LAUNCH_EOF'
#!/bin/bash

# PortKill Enhanced Launch Script
# Default ports to monitor
DEFAULT_PORTS="3000,8000,8080,3001,5000,5173"

# Launch PortKill with default settings
exec /Applications/PortKill\ Enhanced.app/Contents/MacOS/PortKill --ports "$DEFAULT_PORTS"
LAUNCH_EOF

chmod +x "/tmp/$APP_NAME/Contents/MacOS/PortKill"

# Create Info.plist
cat > "/tmp/$APP_NAME/Contents/Info.plist" << 'PLIST_EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleExecutable</key>
	<string>PortKill</string>
	<key>CFBundleIdentifier</key>
	<string>com.martinhessmann.portkill.enhanced</string>
	<key>CFBundleName</key>
	<string>PortKill Enhanced</string>
	<key>CFBundleDisplayName</key>
	<string>PortKill Enhanced</string>
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
mkdir -p "/tmp/PortKill.iconset"
magick assets/large.png -resize 16x16 /tmp/PortKill.iconset/icon_16x16.png
magick assets/large.png -resize 32x32 /tmp/PortKill.iconset/icon_16x16@2x.png
magick assets/large.png -resize 32x32 /tmp/PortKill.iconset/icon_32x32.png
magick assets/large.png -resize 64x64 /tmp/PortKill.iconset/icon_32x32@2x.png
magick assets/large.png -resize 128x128 /tmp/PortKill.iconset/icon_128x128.png
magick assets/large.png -resize 256x256 /tmp/PortKill.iconset/icon_128x128@2x.png
magick assets/large.png -resize 256x256 /tmp/PortKill.iconset/icon_256x256.png
magick assets/large.png -resize 512x512 /tmp/PortKill.iconset/icon_256x256@2x.png
magick assets/large.png -resize 512x512 /tmp/PortKill.iconset/icon_512x512.png
magick assets/large.png -resize 1024x1024 /tmp/PortKill.iconset/icon_512x512@2x.png

iconutil -c icns /tmp/PortKill.iconset -o "/tmp/$APP_NAME/Contents/Resources/PortKill.icns"

# Create DMG
echo "📦 Creating DMG..."

# Calculate size needed (app size + overhead)
APP_SIZE=$(du -sm "/tmp/$APP_NAME" | cut -f1)
DMG_SIZE=$((APP_SIZE + 50)) # Add 50MB overhead

# Create temporary DMG
hdiutil create -size "${DMG_SIZE}m" -fs HFS+ -volname "PortKill Enhanced" -attach "/tmp/$DMG_TEMP"

# Copy app to DMG
cp -R "/tmp/$APP_NAME" "/Volumes/PortKill Enhanced/"

# Create Applications folder link
ln -s /Applications "/Volumes/PortKill Enhanced/Applications"

# Create background image (optional - you can add a custom background)
echo "📋 Creating DMG background..."

# Unmount and convert to compressed DMG
hdiutil detach "/Volumes/PortKill Enhanced"
hdiutil convert "/tmp/$DMG_TEMP" -format UDZO -o "$DMG_NAME"

# Clean up
rm -rf "/tmp/$APP_NAME"
rm -rf "/tmp/PortKill.iconset"
rm "/tmp/$DMG_TEMP"

echo ""
echo "✅ DMG Created Successfully!"
echo "============================"
echo "📁 DMG File: $DMG_NAME"
echo "📏 Size: $(du -h "$DMG_NAME" | cut -f1)"
echo ""
echo "🎯 Installation Instructions:"
echo "1. Double-click the DMG file"
echo "2. Drag 'PortKill Enhanced' to Applications folder"
echo "3. Launch from Applications or Spotlight"
echo ""
echo "✨ Features included:"
echo "  • Custom poison bottle icons"
echo "  • Color-coded status system"
echo "  • Fixed macOS stability issues"
echo "  • Enhanced tray menu"
echo ""
echo "Happy coding! 🚀"
