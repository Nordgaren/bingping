use std::fs;
use std::process::{Command, Stdio};
use std::io::{self, Read, BufRead, BufReader};
use std::env;
use clap::Parser;

#[derive(Parser)]
#[command(version, about = "Like ping, but with ASCII art")]
struct Args {
    /// The target host to ping
    host: String,

    /// Number of packets to send (like ping's -c option)
    #[arg(short = 'c', long = "count")]
    count: Option<String>,

    /// Interval between sending packets in seconds (like ping's -i option)
    #[arg(short = 'i', long = "interval")]
    interval: Option<String>,

    /// Wait timeout in seconds (like ping's -W option)
    #[arg(short = 'W', long = "timeout")]
    timeout: Option<String>,
}

fn load_ascii_art() -> String {
    // ANSI color codes
    let pink = "\x1b[95m";  // Bright magenta (pink)
    let reset = "\x1b[0m";  // Reset to default color
    
    // Try different possible locations for the ASCII art file
    let possible_paths = [
        "ascii-art.txt",                                   // Current directory
        "../ascii-art.txt",                                // Parent directory
        &format!("{}/ascii-art.txt", env!("CARGO_MANIFEST_DIR")), // Cargo manifest directory
    ];

    for path in possible_paths {
        if let Ok(art) = fs::read_to_string(path) {
            // Return the art with pink color
            return format!("{}{}{}", pink, art, reset);
        }
    }

    // If we couldn't find the file, return a simple fallback ASCII art
    format!("{}{}{}",
        pink,
        r#"
    *******************
    *  B I N G P I N G *
    *******************
    "#,
        reset
    )
}

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Always display ASCII art
    println!("{}\n", load_ascii_art());

    // Construct ping command
    let mut cmd = Command::new("ping");
    
    // Add the target host
    cmd.arg(&args.host);
    
    // Add optional arguments if provided
    if let Some(count) = &args.count {
        cmd.arg("-c").arg(count);
    }
    
    if let Some(interval) = &args.interval {
        cmd.arg("-i").arg(interval);
    }
    
    if let Some(timeout) = &args.timeout {
        cmd.arg("-W").arg(timeout);
    }
    
    // Configure stdout and stderr
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    // Execute the ping command
    let mut child = cmd.spawn()?;
    
    // Read and display stdout in real-time
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("{}", line),
                Err(e) => eprintln!("Error reading ping output: {}", e),
            }
        }
    }
    
    // Read stderr if needed
    if let Some(mut stderr) = child.stderr.take() {
        let mut err_output = String::new();
        if stderr.read_to_string(&mut err_output).is_ok() && !err_output.is_empty() {
            eprintln!("{}", err_output);
        }
    }
    
    // Wait for the process to complete
    let status = child.wait()?;
    
    // Exit with the same status code as ping
    std::process::exit(status.code().unwrap_or(1));
}
