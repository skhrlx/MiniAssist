use std::io::Result;
use std::time::{Duration, Instant};
use windows::Win32::{
    UI::WindowsAndMessaging::{SYSTEM_METRICS_INDEX, GetSystemMetrics},
    System::Threading::Sleep,
};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Import our modules
mod desktop_capture;
mod fps_counter;
mod aimbot;
mod input;
mod rcs;

use desktop_capture::DesktopCapture;
use fps_counter::FPSCounter;
use aimbot::Aimbot;
use input::{InputHandler, SLEEP_MS, ACTIVE_SLEEP_MS};
use rcs::RecoilControlSystem;

// Constants for main application
const FPS_TARGET: u32 = 1000;     // Target FPS when active
const FRAME_TIME_TARGET: u32 = 1000 / FPS_TARGET; // Target frame time in ms

fn main() -> Result<()> {
    unsafe {
        // Load configuration
        if let Err(e) = aimbot::load_config() {
            println!("Warning: {}", e);
        }

        // Get screen dimensions
        let screen_width = GetSystemMetrics(SYSTEM_METRICS_INDEX(0));
        let screen_height = GetSystemMetrics(SYSTEM_METRICS_INDEX(1));
        let center_x = screen_width / 2;
        let center_y = screen_height / 2;

        println!("╔════════════════════════════════════════════════╗");
        println!("║                   COLOR AIMBOT                 ║");
        println!("╚════════════════════════════════════════════════╝");
        println!("Screen dimensions: {}x{}", screen_width, screen_height);
        println!("Screen center: ({}, {})", center_x, center_y);
        println!("Controls:");
        println!("  • Hold LEFT MOUSE BUTTON to activate aimbot");
        println!("  • Press L to toggle aimbot on/off");
        println!("  • Press + to increase sensitivity");
        println!("  • Press - to decrease sensitivity");
        println!("  • Press F to toggle FPS display on/off");
        println!("  • Recoil control is active when firing (left mouse button)");
        if aimbot::AUTO_SHOOT {
            println!("  • Auto-shoot is enabled (will fire when on target)");
        }
        println!("  • Press Ctrl+C to exit");
        println!("Performance settings:");
        println!("  • Target FPS: {} FPS", FPS_TARGET);
        println!("  • Field of view: {}x{} pixels", aimbot::FOV, aimbot::FOV);
        println!("  • Frame skip: {} (Processing 1 in {} frames)", input::FRAME_SKIP, input::FRAME_SKIP);
        println!("  • Scan resolution: {} (Checking every {}th pixel)", aimbot::SCAN_STEP, aimbot::SCAN_STEP);
        println!("  • Fine scanning: {}", if aimbot::USE_FINE_SCAN { "Enabled" } else { "Disabled" });

        let desktop_capture = DesktopCapture::new(screen_width, screen_height)?;

        // Initialize components
        let mut aimbot = Aimbot::new(screen_width, screen_height);
        let mut fps_counter = FPSCounter::new();
        let mut input_handler = InputHandler::new();
        let rcs = RecoilControlSystem::new();
        
        // Debug variables
        let mut detection_count = 0;
        let mut frame_processed = 0;
        let mut show_fps = false;

        // Start RCS in a separate thread
        let is_running = Arc::new(AtomicBool::new(true));
        let rcs_running = Arc::new(AtomicBool::new(false));
        let rcs_is_running = rcs_running.clone();
        let rcs_thread_running = is_running.clone();
        
        thread::spawn(move || {
            rcs.run(rcs_thread_running, rcs_is_running);
        });

        loop {
            let frame_start = Instant::now();
            fps_counter.increment();

            // Update FPS counter every second if enabled
            if fps_counter.should_update() && show_fps {
                let (current_fps, avg_fps, frame_time) = fps_counter.update();
                println!("FPS: {:.1} | Avg: {:.1} | Frame time: {:.2}ms | Speed: {:.1}", 
                         current_fps, avg_fps, frame_time, aimbot.get_speed());
            } else if fps_counter.should_update() {
                // Still update the counter even if we don't display it
                fps_counter.update();
            }

            // Check for F key to toggle FPS display
            if input_handler.is_fps_toggle_pressed() {
                show_fps = !show_fps;
                println!("FPS display {}", if show_fps { "ENABLED" } else { "DISABLED" });
                Sleep(300); // Prevent multiple toggles
            }

            // Check for L key to toggle aimbot
            if input_handler.is_toggle_pressed() {
                let is_active = aimbot.toggle();
                println!("Aimbot {}", if is_active { "ACTIVATED" } else { "DEACTIVATED" });
                Sleep(300); // Prevent multiple toggles
            }

            // Check if left mouse button is pressed to activate RCS
            let mouse_pressed = input_handler.is_aim_key_pressed();
            if rcs_running.load(Ordering::Relaxed) != mouse_pressed {
                rcs_running.store(mouse_pressed, Ordering::Relaxed);
            }

            // Check for + key to increase sensitivity
            if input_handler.is_increase_speed_pressed() {
                let new_speed = aimbot.increase_speed();
                println!("Sensitivity increased to: {:.1}", new_speed);
                Sleep(100); // Prevent multiple increments too quickly
            }

            // Check for - key to decrease sensitivity
            if input_handler.is_decrease_speed_pressed() {
                let new_speed = aimbot.decrease_speed();
                println!("Sensitivity decreased to: {:.1}", new_speed);
                Sleep(100); // Prevent multiple decrements too quickly
            }

            // Check if aimbot should be active (either toggled on or mouse button held)
            let should_activate = aimbot.is_active() || input_handler.is_aim_key_pressed();
            
            if !should_activate {
                Sleep(SLEEP_MS); // Reduce CPU usage when inactive
                continue;
            }

            // Frame skipping for performance
            if input_handler.should_skip_frame() {
                Sleep(ACTIVE_SLEEP_MS); // Short sleep between skipped frames
                continue;
            }

            frame_processed += 1;

            // Screenshot and process
            let screenshot_result = desktop_capture.capture_screenshot();
            
            if let Err(e) = screenshot_result {
                // Don't print timeout errors as they're expected
                if e.kind() != std::io::ErrorKind::TimedOut {
                    println!("Screenshot capture error: {:?}", e);
                    Sleep(100); // Wait a bit before trying again
                }
                continue;
            }
            
            let screenshot = screenshot_result.unwrap();
            
            // Find target using the optimized target finder
            if let Some((target_x, target_y, closest_distance)) = aimbot.find_target(&screenshot) {
                detection_count += 1;
                
                // Move mouse to target
                let (move_x, move_y) = aimbot.move_to_target(target_x, target_y);
                
                // Auto-shoot if enabled and close enough to target
                aimbot.auto_shoot(closest_distance);
                
                // Debug info - only print occasionally to avoid flooding console
                if fps_counter.frame_count % 30 == 0 && show_fps {
                    println!("Target at ({}, {}) - Moving {} {} - Distance: {:.1}", 
                             target_x, target_y, move_x, move_y, closest_distance);
                }
            }
            
            // Display detection rate every second
            if fps_counter.should_update() && show_fps {
                let detection_rate = if frame_processed > 0 { (detection_count as f64 / frame_processed as f64) * 100.0 } else { 0.0 };
                println!("Detection rate: {:.1}%", detection_rate);
                
                detection_count = 0;
                frame_processed = 0;
            }
            
            // Frame rate control for stable performance
            let frame_end = Instant::now();
            let frame_duration = frame_end.duration_since(frame_start);
            let target_duration = Duration::from_millis(FRAME_TIME_TARGET as u64);
            
            if frame_duration < target_duration {
                // Sleep the remaining time to maintain stable frame rate
                Sleep((target_duration - frame_duration).as_millis() as u32);
            }
        }
    }
}
