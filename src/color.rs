//! ANSI color codes for terminal output
//! Uses standard ANSI colors (0-15) so they adapt to terminal theme

pub const RESET: &str = "\x1b[0m";
pub const PURPLE: &str = "\x1b[35m"; // Color 5: Magenta
pub const GREEN: &str = "\x1b[32m"; // Color 2: Green
pub const RED: &str = "\x1b[31m"; // Color 1: Red
pub const BLUE: &str = "\x1b[34m"; // Color 4: Blue
