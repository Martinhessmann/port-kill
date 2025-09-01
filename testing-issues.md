# GitHub Issues to Create for v1.3 Testing

Copy and create these issues on GitHub to track testing progress:

## Issue 1: Testing - Custom Poison Bottle Icons

**Title**: Testing: Custom Poison Bottle Icons
**Labels**: testing, v1.3, ui
**Description**:

### Testing Requirements
- [ ] Icon appears correctly in system tray on all macOS versions
- [ ] Color changes work (green → orange → red)
- [ ] Icon scales properly on Retina displays
- [ ] Works in both light and dark mode
- [ ] Fallback to generated icon if assets missing

### Test Scenarios
1. Start with 0 processes → verify green icon
2. Start 1-2 processes → verify orange icon
3. Start 3+ processes → verify red icon
4. Test with 10+ processes → verify icon remains clear
5. Switch between light/dark mode while running

---

## Issue 2: Testing - macOS Application Bundle

**Title**: Testing: macOS Application Bundle
**Labels**: testing, v1.3, macos, installer
**Description**:

### Testing Requirements
- [ ] App launches correctly from /Applications/
- [ ] Icon appears in Dock while running
- [ ] Icon appears correctly in Finder
- [ ] Info.plist permissions are correct
- [ ] App can be moved to trash properly

### Test Scenarios
1. Double-click app in /Applications/ → should launch
2. Drag app to Dock → should create shortcut
3. Right-click → Show Package Contents → verify structure
4. Test with Gatekeeper enabled (unsigned app warning)

---

## Issue 3: Testing - DMG Installer

**Title**: Testing: DMG Installer Package
**Labels**: testing, v1.3, installer
**Description**:

### Testing Requirements
- [ ] DMG opens correctly on double-click
- [ ] Drag-and-drop to Applications works
- [ ] Custom background displays properly
- [ ] File size is reasonable (<10MB)
- [ ] Works on macOS 11+

### Test Scenarios
1. Download DMG → double-click → verify window opens
2. Drag PortKill Enhanced to Applications folder
3. Eject DMG and launch from Applications
4. Test upgrade: install over existing version
5. Test on system without admin privileges

---

## Issue 4: Testing - Enhanced Installation Script

**Title**: Testing: One-liner Installation Script
**Labels**: testing, v1.3, installer, cross-platform
**Description**:

### Testing Requirements
- [ ] Detects platform correctly (macOS/Linux/Windows)
- [ ] Installs Rust if missing
- [ ] Adds binaries to PATH
- [ ] Works on fresh system
- [ ] Handles errors gracefully

### Test Scenarios
1. Fresh macOS system: `curl -sSL ... | bash`
2. System with Rust installed but no port-kill
3. System with old version of port-kill
4. Test with restricted permissions
5. Test offline/network issues handling

---

## Issue 5: Testing - Debug Tools

**Title**: Testing: Debug Script and Tools
**Labels**: testing, v1.3, debugging
**Description**:

### Testing Requirements
- [ ] debug-portkill.sh runs without errors
- [ ] Correctly identifies running instances
- [ ] Log analysis works properly
- [ ] Provides useful troubleshooting info
- [ ] RUST_LOG=debug shows debug menu items

### Test Scenarios
1. Run debug script with port-kill running
2. Run debug script with port-kill not running
3. Test with various error conditions
4. Verify debug menu items appear with RUST_LOG=debug
5. Test manual process killing suggestions