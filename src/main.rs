use std::process::{Command, Stdio};
use std::io::{self, Read, BufRead, BufReader};
use clap::Parser;

#[derive(Parser)]
#[command(version, about = "Like ping, but with ASCII art")]
#[cfg(target_os = "linux")]
struct Args {
    /// The target host to ping
    host: String,

    /// Number of packets to send
    #[arg(short = 'c', long = "count")]
    count: Option<String>,

    /// Interval between sending packets in seconds
    #[arg(short = 'i', long = "interval")]
    interval: Option<String>,
    
    /// Wait timeout in seconds
    #[arg(short = 'W', long = "timeout")]
    timeout: Option<String>,
    
    /// Size of the packet to send in bytes
    #[arg(short = 's', long = "size")]
    size: Option<String>,
    
    /// Time to live
    #[arg(short = 't', long = "ttl")]
    ttl: Option<String>,
}

#[derive(Parser)]
#[command(version, about = "Like ping, but with ASCII art")]
#[cfg(target_os = "windows")]
struct Args {
    /// The target host to ping
    host: String,

    /// Number of packets to send 
    #[arg(short = 'n', long = "count")]
    count: Option<String>,

    /// Interval between sending packets in seconds
    #[arg(short = 'w', long = "interval")]
    interval: Option<String>,
    
    /// Send buffer size in bytes
    #[arg(short = 'l', long = "size")]
    size: Option<String>,
    
    /// Time to live
    #[arg(short = 'i', long = "ttl")]
    ttl: Option<String>,
    
    /// Resolve addresses to hostnames
    #[arg(short = 'a', long = "resolve")]
    resolve: bool,
}


fn load_ascii_art() -> String {
    // ANSI color codes
    let pink = "\x1b[95m";  // Bright magenta (pink)
    let reset = "\x1b[0m";  // Reset to default color

    let art = include_str!("../ascii-art.txt");
    
    format!("{}{}{}", pink, art, reset)

}

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Always display ASCII art
    println!("{}\n", load_ascii_art());

    // Construct ping command
    let mut cmd = Command::new("ping");

    #[cfg(target_os = "windows")]
    {
        // Windows ping syntax
        
        // Add count parameter (-n for Windows)
        if let Some(count) = &args.count {
            cmd.arg("-n").arg(count);
        }
        
        // Add interval parameter (-w for Windows, value in milliseconds)
        if let Some(interval) = &args.interval {
            // Convert seconds to milliseconds for Windows
            if let Ok(secs) = interval.parse::<f64>() {
                let ms = (secs * 1000.0).round().to_string();
                cmd.arg("-w").arg(ms);
            } else {
                cmd.arg("-w").arg(interval);
            }
        }
        
        // Add buffer size parameter (-l for Windows)
        if let Some(size) = &args.size {
            cmd.arg("-l").arg(size);
        }
        
        // Add TTL parameter (-i for Windows)
        if let Some(ttl) = &args.ttl {
            cmd.arg("-i").arg(ttl);
        }
        
        // Add resolve addresses flag (-a for Windows)
        if args.resolve {
            cmd.arg("-a");
        }
        
        // Add target host (last parameter for ping)
        cmd.arg(&args.host);
    }

    #[cfg(target_os = "linux")]
    {
        // Linux ping syntax
        
        // Add the target host first
        cmd.arg(&args.host);
        
        // Add optional arguments
        if let Some(count) = &args.count {
            cmd.arg("-c").arg(count);
        }
        
        if let Some(interval) = &args.interval {
            cmd.arg("-i").arg(interval);
        }
        
        if let Some(timeout) = &args.timeout {
            cmd.arg("-W").arg(timeout);
        }
        
        if let Some(size) = &args.size {
            cmd.arg("-s").arg(size);
        }
        
        if let Some(ttl) = &args.ttl {
            cmd.arg("-t").arg(ttl);
        }
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
