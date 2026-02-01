// Updated configuration settings for improved security and performance.

/// Represents application settings for the bot.
pub struct AppSettings {
    // ... other fields ...

    // Removed metrics field that posed potential data privacy issues.
    // metrics: MetricsSettings,

    // Minimum delay for triggering bot actions, updated for enhanced throttle control.
    pub trigger_bot_delay_min: u32,

    // Maximum delay for triggering bot actions, adjusted for more extensive interval.
    pub trigger_bot_delay_max: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // ... other defaults ...
            trigger_bot_delay_min: 180, // Increased default for better load management.
            trigger_bot_delay_max: 250, // Extended maximum to reduce rapid triggering.
        }
    }
}