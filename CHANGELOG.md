# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - v1.3.0

### Added
- **Custom Poison Bottle Icons** - Dynamic poison bottle icons with color-coded status (green/orange/red based on process count)
  - ⚠️ **Testing Required**: Visual appearance on different macOS versions, light/dark mode compatibility
- **macOS Application Bundle** - Professional .app bundle structure for `/Applications/` installation
  - ⚠️ **Testing Required**: Launch behavior, permissions, icon display in Finder/Dock
- **DMG Installer** - Drag-and-drop installer with custom background and Applications shortcut
  - ✅ **Implemented**: Auto-overwrite functionality for seamless upgrades
  - ⚠️ **Testing Required**: Installation flow, code signing warnings, upgrade from previous versions
- **Enhanced Installation Script** - One-liner installation with automatic dependency handling
  - ⚠️ **Testing Required**: Fresh system installation, PATH configuration, cross-platform detection
- **Comprehensive Debug Tools** - `debug-portkill.sh` script with diagnostics and troubleshooting
  - ⚠️ **Testing Required**: Debug script functionality, log analysis accuracy
- **DEBUGGING.md** - Comprehensive debugging guide with common issues and solutions
- **Enhanced Logging** - Debug menu items and verbose console output when `RUST_LOG=debug`

### Changed
- **Icon System** - Migrated from generic icons to custom poison bottle design
- **Installation Process** - Added multiple installation methods (DMG, one-liner, manual)
- **Documentation** - Comprehensive guides for installation and debugging

### Fixed
- macOS crash issues with event loop handling
- Process detection reliability improvements
- **DMG Creation Issues** - Fixed volume unmounting problems and build script reliability
  - ✅ **Resolved**: Force unmount functionality and Finder window cleanup
  - ✅ **Resolved**: Auto-overwrite existing installations in /Applications/
- **Menu ID Mapping** - Partially fixed hardcoded mapping for common MenuIds
  - ✅ **Works**: Kill All (ID: 10), Quit (ID: 16), Process items (IDs: 12-15)
  - ⚠️ **Issue**: IDs may change between runs

### Known Issues
- **Menu Update Delay** - 10-second delay before menu shows processes (crash prevention)
- **Post-Kill Crashes** - App segfaults ~5 seconds after ANY successful kill operation
  - Crash occurs when menu tries to update after process count changes
  - Processes ARE killed successfully before crash
- **Tray Stability** - Icon may disappear or become unresponsive  
- **Console Mode Recommended** - More stable than tray mode for production use

### Testing Status

#### 🔴 **Not Yet Tested**
- [ ] Custom poison bottle icons on macOS 11/12/13/14
- [ ] Icon appearance in light/dark mode
- [ ] DMG installer on fresh macOS system
- [ ] DMG installer upgrade path
- [ ] Application bundle permissions and launch behavior
- [ ] One-liner installation on fresh systems
- [ ] Debug script functionality
- [ ] Cross-platform installation detection

#### 🟡 **Partially Tested**
- [x] Basic icon generation code compiles
- [x] DMG creation script runs successfully
- [x] Installation script basic functionality
- [x] Auto-overwrite functionality in DMG script

#### 🟢 **Fully Tested**
- [x] Core port monitoring functionality
- [x] Process killing mechanisms
- [x] Console mode operation

## [1.2.0] - Previous Release

### Added
- Docker container detection and display
- PID display option (`--show-pid`)
- Ignore lists for ports and processes

### Changed
- Improved process detection accuracy
- Enhanced error handling

### Fixed
- Memory leaks in process monitoring
- Race conditions in menu updates