# 🐛 PortKill Enhanced Debugging Guide

## Quick Debug Commands

### 1. **Check if PortKill is running**
```bash
ps aux | grep port-kill
```

### 2. **Run in verbose console mode**
```bash
port-kill-console --console --ports 3000,8000 --verbose
```

### 3. **Run with debug logging**
```bash
RUST_LOG=debug port-kill --console --ports 3000,8000
```

### 4. **Test process killing manually**
```bash
# Find processes on a port
lsof -i :3000 -sTCP:LISTEN

# Kill a process manually
lsof -ti :3000 | xargs kill -9
```

### 5. **Check system logs**
```bash
# System logs
log show --predicate 'process == "port-kill"' --last 1h

# Console.app logs
log show --predicate 'process == "port-kill"' --last 1h --info
```

## Common Issues and Solutions

### 🚫 **"Quit" doesn't work**
**Symptoms**: Clicking "Quit" in the menu does nothing

**Debug Steps**:
1. Run in console mode to see what's happening:
   ```bash
   port-kill-console --console --ports 3000,8000 --verbose
   ```
2. Check if the app has proper permissions
3. Look for error messages in the console output
4. Try force-quitting: `pkill -f port-kill`

**Solutions**:
- Restart the app
- Check macOS permissions in System Preferences > Security & Privacy
- Use console mode instead of system tray mode

### 🔪 **Port killing doesn't work**
**Symptoms**: Clicking "Kill" on a process does nothing

**Debug Steps**:
1. Check if you have permission to kill the process:
   ```bash
   lsof -i :3000 -sTCP:LISTEN
   ```
2. Try killing manually:
   ```bash
   lsof -ti :3000 | xargs kill -9
   ```
3. Check if the process is owned by another user

**Solutions**:
- Run with sudo if needed (not recommended for security)
- Check if the process is a system process that can't be killed
- Use console mode for better error reporting

### 👻 **System tray icon doesn't appear**
**Symptoms**: No icon in the menu bar

**Debug Steps**:
1. Check if the app is running:
   ```bash
   ps aux | grep port-kill
   ```
2. Check system tray processes:
   ```bash
   ps aux | grep tray
   ```
3. Look for error messages in console mode

**Solutions**:
- Restart the app
- Check if another instance is running
- Use console mode instead
- Check macOS permissions

### 🔄 **App crashes or freezes**
**Symptoms**: App stops responding or crashes

**Debug Steps**:
1. Check system logs:
   ```bash
   log show --predicate 'process == "port-kill"' --last 1h
   ```
2. Run with debug logging:
   ```bash
   RUST_LOG=debug port-kill --console --ports 3000,8000
   ```
3. Check for memory issues:
   ```bash
   top -pid $(pgrep -f port-kill)
   ```

**Solutions**:
- Restart the app
- Check system resources
- Update to latest version
- Report issue with debug logs

## Advanced Debugging

### **Full Debug Mode**
```bash
# Set maximum debug logging
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Run with all debug info
port-kill-console --console --ports 3000,8000 --verbose
```

### **Process Monitoring Debug**
```bash
# Monitor what PortKill is doing
sudo dtrace -n 'pid$target::*kill*:entry { printf("%s called kill\n", probefunc); }' -p $(pgrep -f port-kill)
```

### **Network Debug**
```bash
# Monitor network connections
sudo lsof -i -P | grep LISTEN

# Check specific ports
netstat -an | grep LISTEN
```

## Debug Script

Run the included debug script for comprehensive diagnostics:
```bash
./debug-portkill.sh
```

This script will:
- Check if PortKill is running
- Verify binaries and permissions
- Test port monitoring
- Check system logs
- Provide troubleshooting recommendations

## Reporting Issues

When reporting issues, include:
1. macOS version: `sw_vers -productVersion`
2. PortKill version: `port-kill --version`
3. Debug output: `RUST_LOG=debug port-kill --console --ports 3000,8000`
4. System logs: `log show --predicate 'process == "port-kill"' --last 1h`
5. Steps to reproduce the issue
