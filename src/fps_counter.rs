use std::time::{Duration, Instant};

// Constants for FPS measurement
pub const FPS_UPDATE_INTERVAL: Duration = Duration::from_millis(1000); // Update FPS display every second
pub const FPS_HISTORY_SIZE: usize = 5; // Number of FPS samples to keep for averaging

// Struct to handle FPS measurement
pub struct FPSCounter {
    pub frame_count: u32,
    last_update: Instant,
    fps_history: Vec<f64>,
    avg_fps: f64,
}

impl FPSCounter {
    // Create a new FPSCounter
    pub fn new() -> Self {
        FPSCounter {
            frame_count: 0,
            last_update: Instant::now(),
            fps_history: Vec::with_capacity(FPS_HISTORY_SIZE),
            avg_fps: 0.0,
        }
    }

    // Increment frame counter
    pub fn increment(&mut self) {
        self.frame_count += 1;
    }

    // Check if it's time to update FPS stats
    pub fn should_update(&self) -> bool {
        Instant::now().duration_since(self.last_update) >= FPS_UPDATE_INTERVAL
    }

    // Update FPS statistics
    pub fn update(&mut self) -> (f64, f64, f64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        let current_fps = self.frame_count as f64 / elapsed;
        
        // Update rolling average of FPS
        self.fps_history.push(current_fps);
        if self.fps_history.len() > FPS_HISTORY_SIZE {
            self.fps_history.remove(0);
        }
        
        // Calculate average FPS
        self.avg_fps = self.fps_history.iter().sum::<f64>() / self.fps_history.len() as f64;
        
        let frame_time = 1000.0 / current_fps; // Convert to milliseconds
        
        // Reset counters
        self.frame_count = 0;
        self.last_update = now;
        
        (current_fps, self.avg_fps, frame_time)
    }
} 