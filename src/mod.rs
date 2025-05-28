// Declare modules
pub mod desktop_capture;
pub mod fps_counter;
pub mod aimbot;
pub mod input;

// Re-export commonly used items
pub use desktop_capture::DesktopCapture;
pub use fps_counter::FPSCounter;
pub use aimbot::Aimbot;
pub use input::InputHandler; 