pub fn bytes_to_human_readable(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    format!("{:.2} {}", size, UNITS[unit_index])
}

pub fn us_to_human_readable(us: u128) -> String {
    if us < 1_000 {
        format!("{} us", us)
    } else if us < 1_000_000 {
        format!("{:.2} ms", us as f64 / 1_000.0)
    } else if us < 60_000_000 {
        format!("{:.2} s", us as f64 / 1_000_000.0)
    } else {
        format!("{:.2} min", us as f64 / 60_000_000.0)
    }
}

pub struct Timer {
    start: std::time::Instant,
    end: Option<std::time::Instant>,
    is_running: bool,
    pub duration_us: u128,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
            end: None,
            is_running: false,
            duration_us: 0,
        }
    }

    pub fn start(&mut self) {
        if !self.is_running {
            self.start = std::time::Instant::now();
            self.is_running = true;
        }
    }

    pub fn stop(&mut self) {
        if self.is_running {
            self.end = Some(std::time::Instant::now());
            self.is_running = false;
            if let Some(end_time) = self.end {
                self.duration_us += end_time.duration_since(self.start).as_micros();
            }
        }
    }
}
