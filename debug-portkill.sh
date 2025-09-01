#!/bin/bash

# 🐛 PortKill Enhanced Debug Script
# Helps debug issues with PortKill functionality

set -e

echo "🐛 PortKill Enhanced Debug Script"
echo "================================"
echo ""

# Check if PortKill is running
echo "🔍 Checking if PortKill is running..."
if pgrep -f "port-kill" > /dev/null; then
    echo "✅ PortKill process found:"
    ps aux | grep port-kill | grep -v grep
else
    echo "❌ PortKill is not running"
fi

echo ""

# Check system tray processes
echo "🔍 Checking system tray processes..."
if pgrep -f "tray" > /dev/null; then
    echo "✅ System tray processes found:"
    ps aux | grep tray | grep -v grep
else
    echo "❌ No system tray processes found"
fi

echo ""

# Check for port-kill binaries
echo "🔍 Checking PortKill binaries..."
if [ -f "/Users/martinhessmann/.local/bin/port-kill" ]; then
    echo "✅ port-kill binary found: $(ls -la /Users/martinhessmann/.local/bin/port-kill)"
else
    echo "❌ port-kill binary not found"
fi

if [ -f "/Users/martinhessmann/.local/bin/port-kill-console" ]; then
    echo "✅ port-kill-console binary found: $(ls -la /Users/martinhessmann/.local/bin/port-kill-console)"
else
    echo "❌ port-kill-console binary not found"
fi

echo ""

# Check Applications folder
echo "🔍 Checking Applications folder..."
if [ -d "/Applications/PortKill.app" ]; then
    echo "✅ PortKill.app found in Applications"
    ls -la "/Applications/PortKill.app/Contents/MacOS/"
else
    echo "❌ PortKill.app not found in Applications"
fi

echo ""

# Test port monitoring
echo "🔍 Testing port monitoring..."
echo "Testing ports 3000, 8000, 8080:"
lsof -i :3000 -sTCP:LISTEN 2>/dev/null || echo "No processes on port 3000"
lsof -i :8000 -sTCP:LISTEN 2>/dev/null || echo "No processes on port 8000"
lsof -i :8080 -sTCP:LISTEN 2>/dev/null || echo "No processes on port 8080"

echo ""

# Check permissions
echo "🔍 Checking permissions..."
echo "Current user: $(whoami)"
echo "User groups: $(groups)"

echo ""

# Test process killing capability
echo "🔍 Testing process killing capability..."
TEST_PID=$$
echo "Testing with current shell PID: $TEST_PID"
if kill -0 $TEST_PID 2>/dev/null; then
    echo "✅ Can send signals to processes"
else
    echo "❌ Cannot send signals to processes"
fi

echo ""

# Check system logs
echo "🔍 Checking system logs for PortKill..."
echo "Recent system.log entries:"
log show --predicate 'process == "port-kill"' --last 1h 2>/dev/null || echo "No recent port-kill entries in system.log"

echo ""

# Check Console.app logs
echo "🔍 Checking Console.app logs..."
echo "Recent Console.app entries:"
log show --predicate 'process == "port-kill"' --last 1h --info 2>/dev/null || echo "No recent port-kill entries in Console.app"

echo ""

# Test verbose mode
echo "🔍 Testing verbose mode..."
echo "Starting PortKill in verbose mode for 10 seconds..."
timeout 10s /Users/martinhessmann/.local/bin/port-kill --console --ports 3000,8000 --verbose 2>&1 || echo "Verbose test completed"

echo ""

# Check for conflicting processes
echo "🔍 Checking for conflicting processes..."
echo "Processes that might conflict:"
ps aux | grep -E "(tray|menu|status)" | grep -v grep || echo "No conflicting processes found"

echo ""

# System information
echo "🔍 System information..."
echo "macOS version: $(sw_vers -productVersion)"
echo "Architecture: $(uname -m)"
echo "Rust version: $(rustc --version 2>/dev/null || echo 'Rust not installed')"

echo ""
echo "🐛 Debug Summary"
echo "==============="
echo "If you're having issues:"
echo "1. Try running in console mode: port-kill-console --console --ports 3000,8000 --verbose"
echo "2. Check if the app has necessary permissions"
echo "3. Restart the app and try again"
echo "4. Check system logs for error messages"
echo "5. Try killing processes manually: lsof -ti :PORT | xargs kill -9"
echo ""
echo "For more detailed debugging, run:"
echo "RUST_LOG=debug port-kill --console --ports 3000,8000"
