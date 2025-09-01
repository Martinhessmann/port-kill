use crate::types::{ProcessInfo, StatusBarInfo};
use anyhow::Result;
use crossbeam_channel::Sender;
use image;
use log::debug;
use std::collections::HashMap;
use std::path::Path;
#[cfg(target_os = "macos")]
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, MenuId, PredefinedMenuItem},
    Icon,
};

#[cfg(target_os = "macos")]
#[derive(Clone)]
pub struct TrayMenu {
    pub icon: Icon,
    menu_sender: Sender<MenuEvent>,
    current_processes: HashMap<u16, ProcessInfo>,
    show_pid: bool,
}

#[cfg(target_os = "macos")]
impl TrayMenu {
    pub fn new(menu_sender: Sender<MenuEvent>) -> Result<Self> {
        // Create a simple icon (we'll use a text-based approach for now)
        let icon = Self::create_icon("0")?;

        // Set up menu event handling
        let sender_clone = menu_sender.clone();
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = sender_clone.send(event);
        }));

        Ok(Self {
            icon,
            menu_sender,
            current_processes: HashMap::new(),
            show_pid: false,
        })
    }

    pub fn update_menu(&mut self, processes: &HashMap<u16, ProcessInfo>, show_pid: bool) -> Result<()> {
        debug!("Updating menu with {} processes", processes.len());

        // Update internal state
        self.current_processes = processes.clone();
        self.show_pid = show_pid;

        Ok(())
    }

    pub fn get_current_menu(&self) -> Result<Menu> {
        Self::create_menu(&self.current_processes, self.show_pid)
    }

    pub fn update_status(&mut self, status_info: &StatusBarInfo) -> Result<()> {
        debug!("Updating status bar: {}", status_info.text);

        // Update icon with new status text
        self.icon = Self::create_icon(&status_info.text)?;

        Ok(())
    }

    pub fn create_menu(processes: &HashMap<u16, ProcessInfo>, show_pid: bool) -> Result<Menu> {
        let menu = Menu::new();

        // Add "Kill All Processes" item with explicit string ID
        let kill_all_item = MenuItem::with_id(
            MenuId("kill_all".to_string()),
            "🔪 Kill All Processes",
            true,
            None,
        );
        menu.append(&kill_all_item)?;

        // Add separator
        let separator = PredefinedMenuItem::separator();
        menu.append(&separator)?;

        // Add individual process items with better organization
        let mut process_entries: Vec<_> = processes.iter().collect();
        // Sort by port for consistent ordering
        process_entries.sort_by_key(|(port, _)| **port);

                 for (_index, (port, process_info)) in process_entries.iter().enumerate() {
            let menu_text = if let (Some(_container_id), Some(container_name)) = (&process_info.container_id, &process_info.container_name) {
                format!(
                    "Kill: Port {}: {} [Docker: {}]",
                    port, process_info.name, container_name
                )
            } else if show_pid {
                format!(
                    "Kill: Port {}: {} (PID {})",
                    port, process_info.name, process_info.pid
                )
            } else {
                format!(
                    "Kill: Port {}: {}",
                    port, process_info.name
                )
            };

            // Create process menu item with string ID for reliable mapping
            let process_item = MenuItem::with_id(
                MenuId(format!("kill_{}", port)),
                &menu_text,
                true,
                None,
            );
            menu.append(&process_item)?;
        }

        // Add another separator if there are processes
        if !processes.is_empty() {
            let separator = PredefinedMenuItem::separator();
            menu.append(&separator)?;
        }

        // Add "Quit" item with explicit string ID
        let quit_item = MenuItem::with_id(
            MenuId("quit".to_string()),
            "❌ Quit",
            true,
            None,
        );
        menu.append(&quit_item)?;

        // Add debug info in debug mode
        if std::env::var("RUST_LOG").unwrap_or_default().contains("debug") {
            let debug_separator = PredefinedMenuItem::separator();
            menu.append(&debug_separator)?;

            let debug_info = MenuItem::new(&format!("Debug: {} processes", processes.len()), false, None);
            menu.append(&debug_info)?;
        }

        Ok(menu)
    }

    // Helper function to get menu item mapping for better debugging
    pub fn get_menu_item_mapping(processes: &HashMap<u16, ProcessInfo>) -> HashMap<String, String> {
        let mut mapping = HashMap::new();

        // Kill All is always first (ID 0 or 10)
        mapping.insert("0".to_string(), "Kill All Processes".to_string());
        mapping.insert("10".to_string(), "Kill All Processes".to_string());

        // Quit is always last (ID 1 or 16)
        let quit_id = if processes.is_empty() { "1" } else { "16" };
        mapping.insert(quit_id.to_string(), "Quit".to_string());

        // Map process items
        let mut process_entries: Vec<_> = processes.iter().collect();
        process_entries.sort_by_key(|(port, _)| **port);

                          for (index, (port, process_info)) in process_entries.iter().enumerate() {
             let menu_text = format!("Kill: Port {}: {}", port, process_info.name);
             let menu_id = if index == 0 { "2" } else if index == 1 { "3" } else if index == 2 { "4" } else { "5" };
            mapping.insert(menu_id.to_string(), menu_text);
        }

        mapping
    }

    pub fn create_icon(text: &str) -> Result<Icon> {
        // Always use the poison bottle icon (custom PNG files are handled within create_poison_bottle_icon)
        Self::create_poison_bottle_icon(text)
    }

    fn load_custom_png_icon(text: &str) -> Result<Icon> {
        // Parse the number to determine which PNG to use
        let number = text.chars().filter(|c| c.is_numeric()).collect::<String>();
        let num = number.parse::<u32>().unwrap_or(0);

        // Try multiple paths for PNG files (app bundle and development)
        let png_paths = if num == 0 {
            vec![
                "assets/green-bottle-36.png",                                    // Development path
                "../Resources/assets/green-bottle-36.png",                      // App bundle path
                "/Applications/PortKill.app/Contents/Resources/assets/green-bottle-36.png", // Absolute app bundle path
                "assets/green-bottle-22.png",                                    // Fallback to 22px
                "../Resources/assets/green-bottle-22.png",
                "/Applications/PortKill.app/Contents/Resources/assets/green-bottle-22.png"
            ]
        } else {
            vec![
                "assets/orange-bottle-36.png",                                   // Development path
                "../Resources/assets/orange-bottle-36.png",                     // App bundle path
                "/Applications/PortKill.app/Contents/Resources/assets/orange-bottle-36.png", // Absolute app bundle path
                "assets/orange-bottle-22.png",                                   // Fallback to 22px
                "../Resources/assets/orange-bottle-22.png",
                "/Applications/PortKill.app/Contents/Resources/assets/orange-bottle-22.png"
            ]
        };

        // Try each path until we find one that works
        for png_path in &png_paths {
            if Path::new(png_path).exists() {
                debug!("Loading PNG file: {}", png_path);

                // Load and decode the PNG file
                match image::open(png_path) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let width = img.width();
                        let height = img.height();

                        debug!("PNG decoded: {}x{} pixels, {} bytes", width, height, rgba.len());

                        // Create icon from RGBA data
                        match Icon::from_rgba(rgba.into_raw(), width, height) {
                            Ok(icon) => {
                                debug!("Successfully created icon from PNG data");
                                return Ok(icon);
                            },
                            Err(e) => {
                                debug!("Failed to create icon from PNG data: {}", e);
                                // Continue to next path or fallback
                            }
                        }
                    },
                    Err(e) => {
                        debug!("Failed to load PNG {}: {}", png_path, e);
                        // Continue to next path
                    }
                }
            }
        }

        Err(anyhow::anyhow!("PNG files not found or PNG decoding not implemented"))
    }

    fn create_poison_bottle_icon(text: &str) -> Result<Icon> {
        // Try to load custom PNG files first
        if let Ok(icon) = Self::load_custom_png_icon(text) {
            return Ok(icon);
        }

        // Generate poison bottle icon with status colors
        let icon_data = Self::generate_poison_bottle_icon(text);

        // Try the actual PNG dimensions first, then fallback to other sizes
        match Icon::from_rgba(icon_data.clone(), 22, 22) {
            Ok(icon) => Ok(icon),
            Err(_) => {
                // Try 16x16 as fallback (common status bar size)
                match Icon::from_rgba(icon_data.clone(), 16, 16) {
                    Ok(icon) => Ok(icon),
                    Err(_) => {
                        // Final fallback to 32x32
                        Icon::from_rgba(icon_data, 32, 32)
                            .map_err(|e| anyhow::anyhow!("Failed to create poison bottle icon: {}", e))
                    }
                }
            }
        }
    }

    fn generate_poison_bottle_icon(text: &str) -> Vec<u8> {
        // Try to load the actual SVG files first
        if let Ok(icon_data) = Self::load_svg_icon(text) {
            return icon_data;
        }

        // Fallback: Create a much simpler, cleaner icon that doesn't try to recreate the complex SVG
        let mut icon_data = Vec::new();
        let size = 22; // Match the status bar appropriate size

        debug!("Generating {}x{} RGBA bitmap = {} bytes", size, size, size * size * 4);

        for y in 0..size {
            for x in 0..size {
                // Parse the number from text to determine status
                let number = text.chars().filter(|c| c.is_numeric()).collect::<String>();
                let num = number.parse::<u32>().unwrap_or(0);

                // Use the exact colors from your SVG files but with a simple, clean design
                let (status_r, status_g, status_b) = if num == 0 {
                    (95, 249, 57) // Green from your green bottle.svg (#5FF939)
                } else {
                    (255, 165, 0) // Orange from your orange bottle.svg (#FFA500)
                };

                // Create a simple, clean circle icon instead of trying to recreate the complex bottle
                let center_x = size as f32 / 2.0;
                let center_y = size as f32 / 2.0;
                let radius = (size as f32 / 2.0) - 2.0; // Leave 2px border

                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();

                let (r, g, b, a) = if distance <= radius {
                    // Solid circle with status color
                    (status_r, status_g, status_b, 255)
                } else {
                    // Transparent background
                    (0, 0, 0, 0)
                };

                icon_data.extend_from_slice(&[r, g, b, a]);
            }
        }

        icon_data
    }

    fn load_svg_icon(text: &str) -> Result<Vec<u8>> {
        // Parse the number to determine which SVG to use
        let number = text.chars().filter(|c| c.is_numeric()).collect::<String>();
        let num = number.parse::<u32>().unwrap_or(0);

        let svg_path = if num == 0 {
            "assets/green bottle.svg"
        } else {
            "assets/orange bottle.svg"
        };

        if Path::new(svg_path).exists() {
            debug!("Found SVG file: {}, but SVG rendering not yet implemented", svg_path);
            // TODO: Implement proper SVG to bitmap conversion using resvg crate
            // For now, this will always fail and use the clean circle fallback
            // The SVG files are perfect 24x24 but we need SVG->bitmap conversion
        }

        Err(anyhow::anyhow!("SVG loading not implemented, using pixel fallback"))
    }

}
