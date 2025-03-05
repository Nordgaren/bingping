// Load ASCII art from file
pub fn load_ascii_art() -> String {
    include_str!("../ascii-art.txt").to_string()
}

// Helper function to format text with rainbow colors
pub fn rainbow_text(text: &str) -> String {
    let colors = [
        "\x1b[38;5;196m", // Red
        "\x1b[38;5;202m", // Orange
        "\x1b[38;5;226m", // Yellow
        "\x1b[38;5;46m",  // Green
        "\x1b[38;5;21m",  // Blue
        "\x1b[38;5;93m",  // Indigo
        "\x1b[38;5;163m", // Violet
    ];
    
    let reset = "\x1b[0m";
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        // Get the color index based on line number
        let color_idx = (i / 2) % colors.len();
        result.push_str(colors[color_idx]);
        result.push_str(line);
        result.push_str(reset);
        
        // Add newline if this isn't the last line
        if i < lines.len() - 1 {
            result.push('\n');
        }
    }
    
    result
}

// Format text with pink color
pub fn pink_text(text: &str) -> String {
    format!("\x1b[38;5;213m{}\x1b[0m", text)
} 