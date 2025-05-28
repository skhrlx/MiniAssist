use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use winapi::um::winuser::{mouse_event, MOUSEEVENTF_MOVE};

/// RecoilControlSystem that moves the mouse down by 1 pixel in a separate thread
pub struct RecoilControlSystem {
    recoil_amount: u32,
    delay_ms: u64,
}

impl RecoilControlSystem {
    /// Create a new RecoilControlSystem with default settings
    pub fn new() -> Self {
        Self {
            recoil_amount: 1,  // Move down by 1 pixel
            delay_ms: 20,      // Delay between mouse movements (ms)
        }
    }

    /// Run the recoil control system in a loop
    pub fn run(&self, is_running: Arc<AtomicBool>, is_active: Arc<AtomicBool>) {
        while is_running.load(Ordering::Relaxed) {
            // Only apply recoil control when active
            if is_active.load(Ordering::Relaxed) {
                unsafe {
                    // Move mouse down by recoil_amount (Y axis)
                    mouse_event(
                        MOUSEEVENTF_MOVE,
                        0,                  // No X movement
                        self.recoil_amount, // Y movement (positive is down)
                        0,
                        0,
                    );
                }
            }
            
            // Sleep to control frequency
            thread::sleep(Duration::from_millis(self.delay_ms));
        }
    }
} 