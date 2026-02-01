// Updated AppSettings struct

struct AppSettings {
    // Other fields...

    // Removed metrics field 

    // Default trigger bot delays updated
    pub default_trigger_bot_delay_min: u32, // Changed from 10ms
    pub default_trigger_bot_delay_max: u32, // Changed from 20ms
}

impl AppSettings {
    pub fn new() -> Self {
        Self {
            // Other fields...

            // Set new human-like default delays
            default_trigger_bot_delay_min: 180,
            default_trigger_bot_delay_max: 250,
        }
    }
}