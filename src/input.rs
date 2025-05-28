use winapi::um::winuser::GetAsyncKeyState;

// Constants for input handling
pub const SLEEP_MS: u32 = 16;         // ~60 FPS when inactive (16ms)
pub const ACTIVE_SLEEP_MS: u32 = 0;   // No sleep when active
pub const FRAME_SKIP: u32 = 1;        // Process every frame for better detection

// Struct to handle input
pub struct InputHandler {
    frame_skip_counter: u32,
}

impl InputHandler {
    // Create a new InputHandler
    pub fn new() -> Self {
        InputHandler {
            frame_skip_counter: 0,
        }
    }

    // Check if L key is pressed to toggle aimbot
    pub unsafe fn is_toggle_pressed(&self) -> bool {
        (GetAsyncKeyState(0x4C) as u16 & 0x8000u16) != 0
    }

    // Check if left mouse button is pressed to activate aimbot
    pub unsafe fn is_aim_key_pressed(&self) -> bool {
        (GetAsyncKeyState(0x01) as u16 & 0x8000u16) != 0
    }

    // Check if + key is pressed to increase sensitivity
    pub unsafe fn is_increase_speed_pressed(&self) -> bool {
        (GetAsyncKeyState(0xBB) as u16 & 0x8000u16) != 0
    }

    // Check if - key is pressed to decrease sensitivity
    pub unsafe fn is_decrease_speed_pressed(&self) -> bool {
        (GetAsyncKeyState(0xBD) as u16 & 0x8000u16) != 0
    }

    // Check if F key is pressed to toggle FPS display
    pub unsafe fn is_fps_toggle_pressed(&self) -> bool {
        (GetAsyncKeyState(0x46) as u16 & 0x8000u16) != 0
    }

    // Check if R key is pressed to toggle recoil control system
    pub unsafe fn is_rcs_toggle_pressed(&self) -> bool {
        (GetAsyncKeyState(0x52) as u16 & 0x8000u16) != 0
    }

    // Check if we should skip this frame
    pub fn should_skip_frame(&mut self) -> bool {
        self.frame_skip_counter = (self.frame_skip_counter + 1) % FRAME_SKIP;
        self.frame_skip_counter != 0
    }
} 