use clap::Parser;

#[cfg(target_os = "linux")]
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Target host to ping
    pub destination: String,

    /// Number of packets to send
    #[clap(short = 'c', long = "count")]
    pub count: Option<u16>,

    /// Number of bytes to send
    #[clap(short = 's', long = "size")]
    pub size: Option<u16>,

    /// Interval between sending packets (in seconds)
    #[clap(short = 'i', long = "interval")]
    pub interval: Option<f64>,

    /// Timeout before giving up (in seconds)
    #[clap(short = 'W', long = "timeout")]
    pub timeout: Option<f64>,

    /// Time to live
    #[clap(short, long)]
    pub ttl: Option<u8>,

    /// Wait time between pings (in ms)
    #[clap(short = 'p', long = "pattern")]
    pub pattern: Option<String>,

    /// Wait time between pings (in ms)
    #[clap(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Verbose output
    #[clap(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Audible ping
    #[clap(short = 'a', long = "audible")]
    pub audible: bool,

    /// Bypass route using socket options
    #[clap(short = 'b')]
    pub bypass_route: bool,

    /// Numeric output only (no DNS resolution)
    #[clap(short = 'n', long = "numeric")]
    pub numeric: bool,

    /// Timestamp display
    #[clap(short = 'D', long = "timestamp")]
    pub timestamp: bool,

    /// Flood ping
    #[clap(short = 'f', long = "flood")]
    pub flood: bool,
    
    /// Use rainbow colors for ASCII art
    #[clap(short = 'r', long = "rainbow")]
    pub rainbow: bool,
}

#[cfg(target_os = "windows")]
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Target host to ping
    pub destination: String,

    /// Number of packets to send
    #[clap(short = 'n', long = "count")]
    pub count: Option<u16>,

    /// Number of bytes to send
    #[clap(short = 'l', long = "size")]
    pub size: Option<u16>,

    /// Interval between sending packets (in seconds)
    #[clap(short = 'i', long = "interval")]
    pub interval: Option<f64>,

    /// Timeout before giving up (in seconds)
    #[clap(short = 'w', long = "timeout")]
    pub timeout: Option<f64>,

    /// Time to live
    #[clap(short = 'i', long = "ttl")]
    pub ttl: Option<u8>,

    /// Record route
    #[clap(short = 'r', long = "record-route")]
    pub record_route: bool,

    /// Timestamp route
    #[clap(short = 's', long = "timestamp")]
    pub timestamp: bool,

    /// Resolve addresses to hostnames
    #[clap(short = 'a', long = "resolve")]
    pub resolve: bool,
    
    /// Use rainbow colors for ASCII art
    #[clap(long = "rainbow")]
    pub rainbow: bool,
} 