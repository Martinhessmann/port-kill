# 🐛 PortKill Enhanced Debugging Guide

## ⚠️ Known Issues (v1.3)

1. **10-second menu update delay** - Menu shows empty until timer expires
2. **App crashes after killing** - Processes ARE killed but app segfaults  
3. **MenuId mapping** - Hardcoded IDs may change between runs
4. **Tray icon sometimes disappears** - macOS tray stability issues

**Recommendation**: Use console mode (`port-kill-console --console`) for stable operation

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

### ⏱️ **Menu shows no processes initially (10-second delay)**
**Symptoms**: 
- Tray icon appears but menu shows empty/no processes
- Menu only updates after 10+ seconds or when clicking the icon
- Log shows: "Process count changed from 0 to X but skipping menu update (killing: false, time passed: false)"

**Root Cause**: 
- Intentional 10-second delay to prevent macOS tray crashes
- Menu won't update until this timer expires

**Debug Steps**:
1. Watch the logs to see when menu updates:
   ```bash
   RUST_LOG=info port-kill --ports 3000,8000 --verbose
   ```
2. Look for: "Process count changed from 0 to X, updating menu..."

**Solutions**:
- Wait 10+ seconds after launch before clicking menu
- Click the tray icon to force refresh
- Use console mode for immediate feedback: `port-kill-console --console`

### 💥 **App crashes after killing ANY process**
**Symptoms**: 
- App crashes with "segmentation fault" after successfully killing processes
- Crash occurs with BOTH "Kill All" AND individual process kills
- Processes ARE killed successfully before the crash
- Crash happens ~5 seconds after kill when menu tries to update

**Debug Steps**:
1. Run with debug logging to capture crash details:
   ```bash
   RUST_LOG=debug port-kill --ports 3000,8000 --verbose
   ```
2. Check if processes were actually killed:
   ```bash
   lsof -ti :3000 :8000 :8080
   ```

**Root Cause**: 
- macOS tray-icon library instability after menu updates
- Known issue with menu recreation after process changes

**Workarounds**:
- Use console mode which doesn't crash: `port-kill-console --console`
- Restart app after kills: `pkill -f port-kill && port-kill`
- Accept that processes ARE killed despite the crash

### 🔢 **Menu actions sometimes don't work (MenuId mapping issue)**
**Symptoms**: 
- Clicking menu items shows "Invalid menu index" or "Unknown menu item" in logs
- MenuIds don't match expected values (e.g., "10" instead of "0" for Kill All)
- Some clicks work, others don't

**Root Cause**: 
- macOS tray-icon crate assigns dynamic MenuIds that change between runs
- Current fix maps common IDs (10=Kill All, 16=Quit, 12-15=Processes)

**Debug Steps**:
1. Check what MenuId was clicked:
   ```bash
   RUST_LOG=info port-kill --ports 3000,8000 --verbose
   # Look for: "Menu ID: XX"
   ```
2. Note which IDs correspond to which menu items

**Solutions**:
- Current hardcoded mapping works for most cases
- If IDs change, update the mapping in app.rs
- Use console mode for reliable operation

### 🔪 **Port killing appears to fail (but actually works)**
**Symptoms**: 
- App crashes after clicking "Kill" 
- Menu doesn't update to show process is gone
- Appears that killing failed

**Reality**: Process killing DOES work - the crash happens afterwards

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
