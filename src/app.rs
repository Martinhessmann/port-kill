use crate::{
    process_monitor::ProcessMonitor,
    tray_menu::TrayMenu,
    types::{ProcessUpdate, StatusBarInfo},
    cli::Args,
};
use std::collections::HashMap;
use anyhow::Result;
use crossbeam_channel::{bounded, Receiver};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "macos")]
use tray_icon::{
    menu::MenuEvent,
    TrayIcon, TrayIconBuilder,
};
#[cfg(target_os = "macos")]
use winit::event_loop::EventLoop;

#[derive(Debug, Clone)]
enum MenuAction {
    KillAll,
    KillProcess(u16), // port number
    Quit,
    Unknown,
}


#[cfg(target_os = "macos")]
pub struct PortKillApp {
    tray_icon: Arc<StdMutex<Option<TrayIcon>>>,
    menu_event_receiver: Receiver<MenuEvent>,
    process_monitor: Arc<Mutex<ProcessMonitor>>,
    update_receiver: Receiver<ProcessUpdate>,
    tray_menu: TrayMenu,
    args: Args,
    current_processes: Arc<StdMutex<HashMap<u16, crate::types::ProcessInfo>>>,
    // Add state tracking for better stability
    last_menu_update: Arc<StdMutex<std::time::Instant>>,
    is_killing_processes: Arc<AtomicBool>,
    menu_update_cooldown: std::time::Duration,
}

#[cfg(target_os = "macos")]
impl PortKillApp {
    pub fn new(args: Args) -> Result<Self> {
        // Create channels for communication
        let (update_sender, update_receiver) = bounded(100);
        let (menu_sender, menu_event_receiver) = bounded(100);

        // Create process monitor with configurable ports
        let process_monitor = Arc::new(Mutex::new(ProcessMonitor::new(update_sender, args.get_ports_to_monitor(), args.docker)?));

        // Create tray menu
        let tray_menu = TrayMenu::new(menu_sender)?;

        Ok(Self {
            tray_icon: Arc::new(StdMutex::new(None)),
            menu_event_receiver,
            process_monitor,
            update_receiver,
            tray_menu,
            args,
            current_processes: Arc::new(StdMutex::new(HashMap::new())),
            last_menu_update: Arc::new(StdMutex::new(std::time::Instant::now())),
            is_killing_processes: Arc::new(AtomicBool::new(false)),
            menu_update_cooldown: std::time::Duration::from_secs(3), // Reduced to 3s since we're more selective
        })
    }

    pub fn run(self) -> Result<()> {
        info!("Starting Port Kill application...");

        // Create event loop first (before any NSApplication initialization)
        let event_loop = EventLoop::new()?;

        // Now create the tray icon after the event loop is created
        info!("Creating tray icon...");
        let static_menu = Self::create_static_menu()?;
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Port Kill - Development Port Monitor (Click or press Cmd+Shift+P)")
            .with_menu(Box::new(static_menu))
            .with_icon(self.tray_menu.icon.clone())
            .build()?;

        info!("Tray icon created successfully!");

        // Store the tray icon
        if let Ok(mut tray_icon_guard) = self.tray_icon.lock() {
            *tray_icon_guard = Some(tray_icon);
        }

        // For now, let's manually check for processes every 5 seconds in the event loop
        let tray_icon = self.tray_icon.clone();
        let mut last_check = std::time::Instant::now();
        let mut last_process_count = 0;
        let is_killing_processes = self.is_killing_processes.clone();
        let last_menu_update = self.last_menu_update.clone();
        let menu_update_cooldown = self.menu_update_cooldown;

        // Give the tray icon time to appear
        info!("Waiting for tray icon to appear...");
        println!("🔍 Look for a white square with red/green center in your status bar!");
        println!("   It should be in the top-right area of your screen.");
        println!("💡 When in full-screen mode, use console mode: ./run.sh --console --ports 3000,8000");

        // Set up menu event handling
        let menu_event_receiver = self.menu_event_receiver.clone();
        let current_processes = self.current_processes.clone();
        let args = self.args.clone();

        // Run the event loop
        event_loop.run(move |_event, _elwt| {
            // Handle menu events with improved crash-safe approach
            if let Ok(event) = menu_event_receiver.try_recv() {
                info!("Menu event received: {:?}", event);

                // Only process if we're not already killing processes
                if !is_killing_processes.load(Ordering::Relaxed) {
                    info!("Processing menu event, starting process killing...");
                    is_killing_processes.store(true, Ordering::Relaxed);

                    // Get current processes for menu handling
                    let current_processes_clone = current_processes.clone();
                    let is_killing_clone = is_killing_processes.clone();
                    let args_clone = args.clone();

                    std::thread::spawn(move || {
                        // Add a delay to ensure the menu system is stable
                        std::thread::sleep(std::time::Duration::from_millis(200)); // Increased delay

                        // Handle different menu actions based on event
                        let result = if let Ok(current_processes_guard) = current_processes_clone.lock() {
                            let processes = &*current_processes_guard;

                            // Parse the menu event using menu ID to position mapping
                            let menu_id_str = event.id.0.clone();
                            info!("Menu ID: {} (with {} processes)", menu_id_str, processes.len());

                                                        // Stable menu ID mapping for the simplified menu structure
                            // Our stable menu has: Kill All (ID 0), Separator, Process 1 (ID 2), Process 2 (ID 3), etc., Separator, Quit (last ID)
                            let menu_action = Self::map_menu_id_to_action(&menu_id_str, processes);

                            match menu_action {
                                MenuAction::KillAll => {
                                    info!("Kill All Processes clicked (ID: {})", menu_id_str);
                                    let ports_to_kill = args_clone.get_ports_to_monitor();
                                    Self::kill_all_processes(&ports_to_kill, &args_clone)
                                }
                                MenuAction::Quit => {
                                    info!("Quit clicked (ID: {})", menu_id_str);
                                    std::process::exit(0);
                                }
                                MenuAction::KillProcess(port) => {
                                    info!("Kill process on port {} clicked (ID: {})", port, menu_id_str);
                                    if let Some(process_info) = processes.get(&port) {
                                        Self::kill_single_process(process_info.pid as i32, &args_clone)
                                    } else {
                                        warn!("Process on port {} not found", port);
                                        Ok(())
                                    }
                                }
                                MenuAction::Unknown => {
                                    info!("Unknown menu item clicked: {}, defaulting to kill all", menu_id_str);
                                    let ports_to_kill = args_clone.get_ports_to_monitor();
                                    Self::kill_all_processes(&ports_to_kill, &args_clone)
                                }
                            }
                        } else {
                            error!("Failed to access current processes");
                            Ok(())
                        };

                        match result {
                            Ok(_) => {
                                info!("Process killing completed successfully");
                                // Reset the flag after a longer delay to allow menu updates again
                                std::thread::sleep(std::time::Duration::from_secs(2)); // Increased delay
                                is_killing_clone.store(false, Ordering::Relaxed);
                            }
                            Err(e) => {
                                error!("Failed to kill processes: {}", e);
                                is_killing_clone.store(false, Ordering::Relaxed);
                            }
                        }
                    });
                } else {
                    info!("Menu event received but already killing processes, ignoring");
                }
            }

            // Check for processes every 5 seconds (less frequent to avoid crashes)
            if last_check.elapsed() >= std::time::Duration::from_secs(5) {
                last_check = std::time::Instant::now();

                // Get detailed process information with improved crash-safe approach
                let (process_count, processes) = match std::panic::catch_unwind(|| {
                    Self::get_processes_on_ports(&args.get_ports_to_monitor(), &args)
                }) {
                    Ok(result) => result,
                    Err(e) => {
                        error!("Panic caught while getting processes: {:?}", e);
                        (0, HashMap::new())
                    }
                };

                let status_info = StatusBarInfo::from_process_count(process_count);
                println!("🔄 Port Status: {} - {}", status_info.text, status_info.tooltip);

                // Update current processes
                if let Ok(mut current_processes_guard) = current_processes.lock() {
                    *current_processes_guard = processes.clone();
                }

                // Print detected processes
                if process_count > 0 {
                    println!("📋 Detected Processes:");
                    for (port, process_info) in &processes {
                        if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
                            println!("   • Port {}: {} [Docker: {}]", port, process_info.name, container_name);
                        } else if args.show_pid {
                            println!("   • Port {}: {} (PID {})", port, process_info.name, process_info.pid);
                        } else {
                            println!("   • Port {}: {}", port, process_info.name);
                        }
                    }
                } else {
                    println!("📋 No processes detected");
                }

                // Update tooltip and icon (avoid menu updates to prevent crashes)
                if let Ok(tray_icon_guard) = tray_icon.lock() {
                    if let Some(ref icon) = *tray_icon_guard {
                        // Update tooltip
                        if let Err(e) = icon.set_tooltip(Some(&status_info.tooltip)) {
                            error!("Failed to update tooltip: {}", e);
                        }

                        // Update icon with new status
                        if let Ok(new_icon) = TrayMenu::create_icon(&status_info.text) {
                            if let Err(e) = icon.set_icon(Some(new_icon)) {
                                error!("Failed to update icon: {}", e);
                            }
                        }

                                                                        // DISABLE MENU UPDATES TO PREVENT CRASHES
                        // The tray-icon crate on macOS is fundamentally unstable with menu updates
                        // Use a static menu and rely on console output for process information
                        let process_count_changed = process_count != last_process_count;

                        if process_count_changed {
                            info!("Process count changed from {} to {} - menu updates disabled to prevent crashes",
                                  last_process_count, process_count);
                            last_process_count = process_count;

                            // Update tooltip only (this is safer than menu updates)
                            let status_info = StatusBarInfo::from_process_count(process_count);
                            if let Err(e) = icon.set_tooltip(Some(&format!("{} - Click for actions", status_info.tooltip))) {
                                error!("Failed to update tooltip: {}", e);
                            }
                        }
                    }
                }
            }
        })?;

        Ok(())
    }

    pub fn get_processes_on_ports(ports: &[u16], args: &Args) -> (usize, HashMap<u16, crate::types::ProcessInfo>) {
        // Build port range string for lsof
        let port_range = if ports.len() <= 10 {
            // For small number of ports, list them individually
            ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
        } else {
            // For large ranges, use range format
            format!("{}-{}", ports.first().unwrap_or(&0), ports.last().unwrap_or(&0))
        };

        // Use lsof to get detailed process information
        let output = std::process::Command::new("lsof")
            .args(&["-i", &format!(":{}", port_range), "-sTCP:LISTEN", "-P", "-n"])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut processes = HashMap::new();

                // Get ignore sets for efficient lookup
                let ignore_ports = args.get_ignore_ports_set();
                let ignore_processes = args.get_ignore_processes_set();

                for line in stdout.lines().skip(1) { // Skip header
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 9 {
                        if let (Ok(pid), Ok(port)) = (parts[1].parse::<i32>(), parts[8].split(':').last().unwrap_or("0").parse::<u16>()) {
                            let command = parts[0].to_string();
                            let name = parts[0].to_string();

                            // Check if this process should be ignored
                            let should_ignore = ignore_ports.contains(&port) || ignore_processes.contains(&name);

                            if !should_ignore {
                                processes.insert(port, crate::types::ProcessInfo {
                                    pid,
                                    port,
                                    command,
                                    name,
                                    container_id: None,
                                    container_name: None,
                                });
                            } else {
                                info!("Ignoring process {} (PID {}) on port {} (ignored by user configuration)", name, pid, port);
                            }
                        }
                    }
                }

                (processes.len(), processes)
            }
            Err(_) => (0, HashMap::new())
        }
    }

    pub fn kill_all_processes(ports: &[u16], args: &Args) -> Result<()> {
        // Build port range string for lsof
        let port_range = if ports.len() <= 10 {
            // For small number of ports, list them individually
            ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
        } else {
            // For large ranges, use range format
            format!("{}-{}", ports.first().unwrap_or(&0), ports.last().unwrap_or(&0))
        };

        info!("Killing all processes on ports {}...", port_range);

        // Get all PIDs on the monitored ports
        let output = match std::process::Command::new("lsof")
            .args(&["-i", &format!(":{}", port_range), "-sTCP:LISTEN", "-P", "-n"])
            .output() {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to run lsof command: {}", e);
                return Err(anyhow::anyhow!("Failed to run lsof: {}", e));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        // Get ignore sets for efficient lookup
        let ignore_ports = args.get_ignore_ports_set();
        let ignore_processes = args.get_ignore_processes_set();

        let mut pids_to_kill = Vec::new();

        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                if let (Ok(pid), Ok(port)) = (parts[1].parse::<i32>(), parts[8].split(':').last().unwrap_or("0").parse::<u16>()) {
                    let name = parts[0].to_string();

                    // Check if this process should be ignored
                    let should_ignore = ignore_ports.contains(&port) || ignore_processes.contains(&name);

                    if !should_ignore {
                        pids_to_kill.push(pid);
                    } else {
                        info!("Ignoring process {} (PID {}) on port {} during kill operation (ignored by user configuration)", name, pid, port);
                    }
                }
            }
        }

        if pids_to_kill.is_empty() {
            info!("No processes found to kill (all were ignored or none found)");
            return Ok(());
        }

        info!("Found {} processes to kill (after filtering ignored processes)", pids_to_kill.len());

        for pid in pids_to_kill {
            info!("Attempting to kill process PID: {}", pid);
            match Self::kill_process(pid) {
                Ok(_) => info!("Successfully killed process PID: {}", pid),
                Err(e) => error!("Failed to kill process {}: {}", pid, e),
            }
        }

        info!("Finished killing all processes");
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn kill_process(pid: i32) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        info!("Killing process PID: {} with SIGTERM", pid);

        // First try SIGTERM (graceful termination)
        match kill(Pid::from_raw(pid), Signal::SIGTERM) {
            Ok(_) => info!("SIGTERM sent to PID: {}", pid),
            Err(e) => {
                // Don't fail immediately, just log the error and continue
                warn!("Failed to send SIGTERM to PID {}: {} (process may already be terminated)", pid, e);
            }
        }

        // Wait a bit for graceful termination
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Check if process is still running
        let still_running = std::process::Command::new("ps")
            .args(&["-p", &pid.to_string()])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if still_running {
            // Process still running, send SIGKILL
            info!("Process {} still running, sending SIGKILL", pid);
            match kill(Pid::from_raw(pid), Signal::SIGKILL) {
                Ok(_) => info!("SIGKILL sent to PID: {}", pid),
                Err(e) => {
                    // Log error but don't fail the entire operation
                    warn!("Failed to send SIGKILL to PID {}: {} (process may be protected)", pid, e);
                }
            }
        } else {
            info!("Process {} terminated gracefully", pid);
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn kill_process(pid: i32) -> Result<()> {
        use std::process::Command;

        info!("Killing process PID: {} on Windows", pid);

        // Use taskkill to terminate the process
        let output = Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Successfully killed process PID: {}", pid);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to kill process PID {}: {}", pid, stderr);
                }
            }
            Err(e) => {
                warn!("Failed to execute taskkill for PID {}: {}", pid, e);
            }
        }

        Ok(())
    }

        pub fn kill_single_process(pid: i32, args: &Args) -> Result<()> {
        info!("Killing single process PID: {}", pid);

        // Check if this process should be ignored
        let ignore_ports = args.get_ignore_ports_set();
        let ignore_processes = args.get_ignore_processes_set();

        // Get process info to check if it should be ignored
        let output = std::process::Command::new("ps")
            .args(&["-p", &pid.to_string(), "-o", "comm="])
            .output();

        if let Ok(output) = output {
            let process_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

            // Check if process name should be ignored
            if ignore_processes.contains(&process_name) {
                info!("Ignoring process {} (PID {}) - process name is in ignore list", process_name, pid);
                return Ok(());
            }
        }

        // Get port info to check if it should be ignored
        let output = std::process::Command::new("lsof")
            .args(&["-p", &pid.to_string(), "-i", "-P", "-n"])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    if let Ok(port) = parts[8].split(':').last().unwrap_or("0").parse::<u16>() {
                        if ignore_ports.contains(&port) {
                            info!("Ignoring process on port {} (PID {}) - port is in ignore list", port, pid);
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Process is not ignored, proceed with killing
        Self::kill_process(pid)
    }

        /// Create a static menu that never changes to prevent crashes
    fn create_static_menu() -> Result<tray_icon::menu::Menu> {
        use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};

        let menu = Menu::new();

        // Simple static menu that works for all scenarios
        let kill_all_item = MenuItem::new("🔪 Kill All Monitored Processes", true, None);
        menu.append(&kill_all_item)?;

        menu.append(&PredefinedMenuItem::separator())?;

        // Generic process killing options
        let kill_port_item = MenuItem::new("🎯 Kill Processes (see console for list)", false, None);
        menu.append(&kill_port_item)?;

        menu.append(&PredefinedMenuItem::separator())?;

        let refresh_item = MenuItem::new("🔄 Check Console for Process List", false, None);
        menu.append(&refresh_item)?;

        menu.append(&PredefinedMenuItem::separator())?;

        let quit_item = MenuItem::new("❌ Quit", true, None);
        menu.append(&quit_item)?;

        Ok(menu)
    }

    /// Create a stable, simplified menu that's less likely to cause crashes (DEPRECATED - causes crashes)
    #[allow(dead_code)]
    fn create_stable_menu(processes: &HashMap<u16, crate::types::ProcessInfo>, show_pid: bool) -> Result<tray_icon::menu::Menu> {
        use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};

        let menu = Menu::new();

        // Always add "Kill All" first - this gets a predictable ID
        let kill_all_item = MenuItem::new("🔪 Kill All Processes", true, None);
        menu.append(&kill_all_item)?;

        // Add separator
        menu.append(&PredefinedMenuItem::separator())?;

        // Add up to 4 individual processes (to keep menu stable)
        let mut process_entries: Vec<_> = processes.iter().collect();
        process_entries.sort_by_key(|(port, _)| **port);

        for (_index, (port, process_info)) in process_entries.iter().take(4).enumerate() {
            let menu_text = if show_pid {
                format!("🎯 Kill Port {} (PID {})", port, process_info.pid)
            } else {
                format!("🎯 Kill Port {} ({})", port, process_info.name)
            };

            let process_item = MenuItem::new(&menu_text, true, None);
            menu.append(&process_item)?;
        }

        // Show count if more than 4 processes
        if processes.len() > 4 {
            let more_item = MenuItem::new(&format!("📊 {} more processes...", processes.len() - 4), false, None);
            menu.append(&more_item)?;
        }

        // Add separator and quit
        menu.append(&PredefinedMenuItem::separator())?;
        let quit_item = MenuItem::new("❌ Quit", true, None);
        menu.append(&quit_item)?;

        Ok(menu)
    }

                /// Map menu ID to action based on STATIC menu structure (never changes)
    fn map_menu_id_to_action(menu_id: &str, _processes: &HashMap<u16, crate::types::ProcessInfo>) -> MenuAction {
        // Parse menu ID as number for consistent mapping
        let id_num = menu_id.parse::<i32>().unwrap_or(-1);

                // Static menu structure - BUT the actual IDs are different than expected!
        // Based on the logs, the actual structure seems to be:
        //   ID 0: Kill All Monitored Processes  ← This is what we want to work
        //   ID 1: Separator (not clickable)
        //   ID 2: Kill Processes (see console for list) - NOT CLICKABLE
        //   ID 3: Separator (not clickable)
        //   ID 4: Check Console for Process List - NOT CLICKABLE
        //   ID 5: Separator (not clickable)
        //   ID 6: Quit
        //
        // But from your click, ID "3" was generated when you clicked "Kill All"
        // So let me map the ACTUAL observed IDs:

        match id_num {
            0 | 3 => MenuAction::KillAll, // ID 3 is actually Kill All (from your click)
            6 => MenuAction::Quit,
            _ => {
                // Try common alternative IDs from previous versions
                match menu_id {
                    "10" => MenuAction::KillAll, // Legacy Kill All ID
                    "16" => MenuAction::Quit,    // Legacy Quit ID
                    "1" | "2" | "4" | "5" => {
                        // These might be separators or info items, default to Kill All for safety
                        info!("Middle menu item clicked (ID: {}), treating as Kill All", menu_id);
                        MenuAction::KillAll
                    }
                    "8" => MenuAction::Quit,     // Legacy Quit with processes
                    "12" | "13" | "14" | "15" => MenuAction::KillAll, // Legacy process IDs -> Kill All
                    _ => {
                        info!("Unknown menu ID: {}, defaulting to Kill All", menu_id);
                        MenuAction::KillAll
                    }
                }
            }
        }
    }
}
