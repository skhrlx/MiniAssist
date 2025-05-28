use winapi::um::winuser::{mouse_event, MOUSEEVENTF_MOVE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP};
use windows::Win32::System::Threading::Sleep;
use std::fs;
use std::path::Path;

// Default values for aimbot (used if config file is missing or has errors)
const DEFAULT_FOV: u32 = 36;
const DEFAULT_GREEN_THRESHOLD: i32 = 40;
const DEFAULT_RED_MAX: i32 = 60;
const DEFAULT_BLUE_MAX: i32 = 60;
const DEFAULT_COLOR_DIFF: i32 = 30;
const DEFAULT_SCAN_STEP: u32 = 2;
const DEFAULT_USE_FINE_SCAN: bool = true;
const DEFAULT_AUTO_SHOOT: bool = false;
const DEFAULT_MOVE_SPEED: f32 = 0.6;
const DEFAULT_SPEED_INCREMENT: f32 = 0.1;

// Constants for aimbot (will be loaded from config)
pub static mut FOV: u32 = DEFAULT_FOV;
pub static mut GREEN_THRESHOLD: i32 = DEFAULT_GREEN_THRESHOLD;
pub static mut RED_MAX: i32 = DEFAULT_RED_MAX;
pub static mut BLUE_MAX: i32 = DEFAULT_BLUE_MAX;
pub static mut COLOR_DIFF: i32 = DEFAULT_COLOR_DIFF;
pub static mut SCAN_STEP: u32 = DEFAULT_SCAN_STEP;
pub static mut USE_FINE_SCAN: bool = DEFAULT_USE_FINE_SCAN;
pub static mut AUTO_SHOOT: bool = DEFAULT_AUTO_SHOOT;
pub static mut MOVE_SPEED: f32 = DEFAULT_MOVE_SPEED;
pub static mut SPEED_INCREMENT: f32 = DEFAULT_SPEED_INCREMENT;
// Function to load config from file
pub fn load_config() -> Result<(), String> {
    let config_path = Path::new("config.txt");
    
    if !config_path.exists() {
        return Err("Config file not found. Using default values.".to_string());
    }

    match fs::read_to_string(config_path) {
        Ok(contents) => {
            for line in contents.lines() {
                if line.trim().is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() != 2 {
                    continue;
                }

                let key = parts[0].trim();
                let value = parts[1].trim();

                unsafe {
                    match key {
                        "FOV" => FOV = value.parse().unwrap_or(DEFAULT_FOV),
                        "GREEN_THRESHOLD" => GREEN_THRESHOLD = value.parse().unwrap_or(DEFAULT_GREEN_THRESHOLD),
                        "RED_MAX" => RED_MAX = value.parse().unwrap_or(DEFAULT_RED_MAX),
                        "BLUE_MAX" => BLUE_MAX = value.parse().unwrap_or(DEFAULT_BLUE_MAX),
                        "COLOR_DIFF" => COLOR_DIFF = value.parse().unwrap_or(DEFAULT_COLOR_DIFF),
                        "SCAN_STEP" => SCAN_STEP = value.parse().unwrap_or(DEFAULT_SCAN_STEP),
                        "USE_FINE_SCAN" => USE_FINE_SCAN = value.parse().unwrap_or(DEFAULT_USE_FINE_SCAN),
                        "AUTO_SHOOT" => AUTO_SHOOT = value.parse().unwrap_or(DEFAULT_AUTO_SHOOT),
                        "DEFAULT_SPEED" => MOVE_SPEED = value.parse().unwrap_or(DEFAULT_MOVE_SPEED),
                        "SPEED_INCREMENT" => SPEED_INCREMENT = value.parse().unwrap_or(DEFAULT_SPEED_INCREMENT),
                        _ => continue,
                    }
                }
            }
            Ok(())
        }
        Err(e) => Err(format!("Error reading config file: {}", e))
    }
}

// Struct to handle aimbot functionality
pub struct Aimbot {
    screen_width: i32,
    screen_height: i32,
    center_x: i32,
    center_y: i32,
    move_speed: f32,
    is_active: bool,
}

impl Aimbot {
    // Create a new Aimbot
    pub fn new(screen_width: i32, screen_height: i32) -> Self {
        let center_x = screen_width / 2;
        let center_y = screen_height / 2;
        
        Aimbot {
            screen_width,
            screen_height,
            center_x,
            center_y,
            move_speed: unsafe { MOVE_SPEED },
            is_active: false,
        }
    }

    // Toggle aimbot activation
    pub fn toggle(&mut self) -> bool {
        self.is_active = !self.is_active;
        self.is_active
    }

    // Increase movement speed
    pub fn increase_speed(&mut self) -> f32 {
        unsafe {
            self.move_speed += SPEED_INCREMENT;
        }
        self.move_speed
    }

    // Decrease movement speed
    pub fn decrease_speed(&mut self) -> f32 {
        unsafe {
            self.move_speed = (self.move_speed - SPEED_INCREMENT).max(0.1);
        }
        self.move_speed
    }

    // Check if aimbot is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    // Get current movement speed
    pub fn get_speed(&self) -> f32 {
        self.move_speed
    }

    // Find target in the screenshot
    pub unsafe fn find_target(&self, screenshot: &Vec<u8>) -> Option<(usize, usize, f32)> {
        // Define scan area (FOV around center)
        let start_x = (self.center_x as i32 - FOV as i32 / 2).max(0) as usize;
        let end_x = (self.center_x as i32 + FOV as i32 / 2).min(self.screen_width - 1) as usize;
        let start_y = (self.center_y as i32 - FOV as i32 / 2).max(0) as usize;
        let end_y = (self.center_y as i32 + FOV as i32 / 2).min(self.screen_height - 1) as usize;
        
        let mut found_green = false;
        let mut closest_distance = FOV as f32;
        let mut target_x = 0;
        let mut target_y = 0;
        
        // First coarse scan - with large step size for better performance
        for y in (start_y..end_y).step_by(SCAN_STEP as usize) {
            for x in (start_x..end_x).step_by(SCAN_STEP as usize) {
                let pitch = self.screen_width as usize * 4; // 4 bytes per pixel (BGRA)
                let index = y * pitch + x * 4;
                
                // Safety check for out of bounds access
                if index + 2 >= screenshot.len() {
                    continue;
                }
                
                // BGRA format
                let b = screenshot[index] as u8;
                let g = screenshot[index + 1] as u8;
                let r = screenshot[index + 2] as u8;
                
                // Fast green detection
                if Self::is_green_pixel(r, g, b) {
                    // Calculate distance from center
                    let dx = x as i32 - self.center_x;
                    let dy = y as i32 - self.center_y;
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    
                    // Update if closer than previous
                    if distance < closest_distance {
                        closest_distance = distance;
                        target_x = x;
                        target_y = y;
                        found_green = true;
                    }
                }
            }
        }
        
        // Fine scan around the initial detection (if enabled and if we found something)
        if found_green && USE_FINE_SCAN {
            // Define a small area around the initial target for fine scanning
            let fine_size = SCAN_STEP as i32 * 2;
            let fine_start_x = (target_x as i32 - fine_size).max(start_x as i32) as usize;
            let fine_end_x = (target_x as i32 + fine_size).min(end_x as i32) as usize;
            let fine_start_y = (target_y as i32 - fine_size).max(start_y as i32) as usize;
            let fine_end_y = (target_y as i32 + fine_size).min(end_y as i32) as usize;
            
            // Scan with step size 1 for precision
            for y in fine_start_y..fine_end_y {
                for x in fine_start_x..fine_end_x {
                    let pitch = self.screen_width as usize * 4;
                    let index = y * pitch + x * 4;
                    
                    // Safety check
                    if index + 2 >= screenshot.len() {
                        continue;
                    }
                    
                    let b = screenshot[index] as u8;
                    let g = screenshot[index + 1] as u8;
                    let r = screenshot[index + 2] as u8;
                    
                    if Self::is_green_pixel(r, g, b) {
                        let dx = x as i32 - self.center_x;
                        let dy = y as i32 - self.center_y;
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        
                        if distance < closest_distance {
                            closest_distance = distance;
                            target_x = x;
                            target_y = y;
                        }
                    }
                }
            }
        }
        
        if found_green {
            Some((target_x, target_y, closest_distance))
        } else {
            None
        }
    }

    // Move mouse to target
    pub unsafe fn move_to_target(&self, target_x: usize, target_y: usize) -> (i32, i32) {
        // Calculate how much to move the mouse
        let dx = (target_x as i32 - self.center_x) as f32;
        let dy = (target_y as i32 - self.center_y) as f32;
        
        // Apply sensitivity to movement
        let move_x = (dx * self.move_speed) as i32;
        let move_y = (dy * self.move_speed) as i32;
        
        // Move mouse
        mouse_event(MOUSEEVENTF_MOVE, move_x as u32, move_y as u32, 0, 0);
        
        (move_x, move_y)
    }

    // Auto-shoot if enabled and close enough to target
    pub unsafe fn auto_shoot(&self, distance: f32) {
        if AUTO_SHOOT && distance < 10.0 {
            mouse_click(false);
        }
    }

    // Helper function to check if a pixel is green based on our thresholds
    #[inline(always)]
    fn is_green_pixel(r: u8, g: u8, b: u8) -> bool {
        let r = r as i32;
        let g = g as i32;
        let b = b as i32;
        
        unsafe {
            g > GREEN_THRESHOLD && 
            r < RED_MAX && 
            b < BLUE_MAX && 
            g > r + COLOR_DIFF && 
            g > b + COLOR_DIFF
        }
    }
}

// Helper function to click the mouse
unsafe fn mouse_click(right: bool) {
    if right {
        mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
        Sleep(10);
        mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
    } else {
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
        Sleep(10);
        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
    }
} 