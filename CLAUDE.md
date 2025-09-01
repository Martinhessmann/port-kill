# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build Commands
```bash
# Platform-specific builds
./build-macos.sh      # macOS with tray icon
./build-linux.sh      # Linux with GTK tray (creates temporary Cargo.toml)
build-windows.bat     # Windows with tray icon

# Generic build (macOS configuration only)
cargo build --release

# Run tests
cargo test
cargo test -- --nocapture    # With output

# Code quality checks
cargo clippy              # Lint code
cargo fmt                 # Format code
cargo fmt -- --check      # Check formatting
```

### Run Commands
```bash
# Platform-specific run scripts
./run.sh              # macOS (defaults to tray mode)
./run-linux.sh        # Linux (falls back to console if GTK missing)
run-windows.bat       # Windows (defaults to tray mode)

# Common options
./run.sh --ports 3000,8000,8080 --docker --verbose
./run.sh --console --log-level warn    # Console mode for SSH/full-screen
./run.sh --show-pid --ignore-ports 5353,5000

# Debug mode
RUST_LOG=debug ./run.sh --verbose
```

### Release Management
```bash
# Create a new release (triggers CI/CD)
./release.sh 0.1.0

# Create macOS DMG installer
./create-dmg.sh
```

## Architecture Overview

Port Kill is a cross-platform system tray application that monitors and manages processes on configurable ports. It provides both GUI (system tray) and console interfaces.

### Multi-Binary Architecture
- **port-kill**: Platform-specific system tray binary
  - macOS: Uses `tray-icon` with native Cocoa integration
  - Linux: Uses `tray-item` with GTK3 backend
  - Windows: Uses `tray-item` with native Windows API
- **port-kill-console**: Cross-platform console binary (no GUI dependencies)

### Core Components

1. **ProcessMonitor** (`process_monitor.rs`)
   - Scans configured ports using platform-specific tools (`lsof` on Unix, `netstat` on Windows)
   - Runs asynchronously with 2-second intervals
   - Detects Docker containers and includes container names
   - Communicates updates via crossbeam channels

2. **Platform-Specific Entry Points**
   - `main.rs` - macOS tray app with `winit` event loop
   - `main_linux.rs` - Linux tray with GTK fallback to console
   - `main_windows.rs` - Windows tray implementation
   - `main_console.rs` - Console mode for all platforms

3. **Tray Management** (macOS: `tray_menu.rs`, Others: inline)
   - Dynamic menu generation based on detected processes
   - Visual status indicators (green/orange/red icons)
   - Individual process killing and bulk operations
   - Platform-specific icon handling

4. **CLI Arguments** (`cli.rs`)
   - Configurable port ranges and specific port lists
   - Docker support, PID display, ignore lists
   - Log level control and verbose mode
   - Validation and error handling

### Process Detection and Termination

- **Unix Systems**: Uses `lsof -ti :PORT -sTCP:LISTEN` for process detection
- **Windows**: Uses `netstat -ano` + `tasklist` for process details
- **Termination Strategy**: 
  - Unix: SIGTERM → wait 500ms → SIGKILL
  - Windows: `taskkill /F`
  - Docker: `docker stop` → `docker rm -f`

### Communication Flow
1. ProcessMonitor detects changes → sends ProcessUpdate via channel
2. Main thread receives update → updates tray icon/menu
3. User clicks menu item → spawns kill task in background
4. Kill task completes → ProcessMonitor detects change on next scan

## Key Implementation Details

### Platform-Specific Build Process

**Linux Build Process**:
- `build-linux.sh` creates temporary `Cargo.linux.tmp.toml` with GTK dependencies
- Creates temporary `lib.linux.tmp.rs` excluding macOS-specific modules
- Backs up and restores original files after build
- Checks for required packages and provides installation instructions

### Platform-Specific Considerations

**macOS**:
- Requires `winit` event loop for tray icon updates
- Uses `tray-icon` crate with native Cocoa backend
- Handles full-screen mode limitations

**Linux**:
- Requires GTK3 packages: `libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev libxdo-dev`
- Automatically falls back to console mode if GTK unavailable
- Uses `tray-item` with GTK backend

**Windows**:
- Uses `tray-item` with native Windows tray API
- Relies on `netstat` and `tasklist` for process information
- No Unix signal handling (uses `taskkill` instead)

### Error Handling Strategy
- Uses `anyhow` for error propagation with context
- `thiserror` for custom error types
- Graceful degradation (e.g., Linux GTK → console fallback)
- Comprehensive logging with `env_logger`

### Testing Approach
- Unit tests for CLI argument parsing
- CI/CD tests binary execution on all platforms
- Console mode tests for cross-platform compatibility
- Platform-specific build verification

## Common Development Tasks

### Adding New Features
1. Update `Args` struct in `cli.rs` for new CLI options
2. Modify `ProcessMonitor` for new detection logic
3. Update platform-specific menu builders for new actions
4. Add corresponding console mode output

### Debugging Issues
```bash
# Enable debug logging
RUST_LOG=debug ./run.sh --verbose

# Run comprehensive debug script
./debug-portkill.sh

# Linux-specific debugging
./debug_linux.sh

# Check process detection manually
lsof -ti :3000 -sTCP:LISTEN      # Unix
netstat -ano | findstr :3000      # Windows
```

### CI/CD Workflow
- **build.yml**: Tests builds on PR/push to main
- **release.yml**: Builds release binaries when release published
- **auto-release.yml**: Creates GitHub release from tags
- **debug.yml**: Additional debugging workflow
- Use `./release.sh X.Y.Z` to trigger full release pipeline

## Visual Design

### Custom Poison Bottle Icons
The app uses dynamically generated poison bottle icons with color-coded status:
- **Green**: 0 processes (safe, no development servers)
- **Orange**: 1-2 processes (some development servers)
- **Red**: 3+ processes (many development servers)

Icon generation: `tray_menu.rs` → `generate_poison_bottle_icon()`

## Important Notes

- Always test both tray and console modes when making changes
- Console mode must work without any GUI dependencies
- Maintain cross-platform compatibility in shared code
- Use platform-specific code only in designated entry points
- Follow Rust idioms and use clippy for linting
- Test new features on all supported platforms before release
- Document any platform-specific behavior or limitations
- The Linux build process requires special handling due to GTK dependencies