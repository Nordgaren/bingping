# BingPing

BingPing is a wrapper around the standard ping utility that adds ASCII art to the output.

## Features

- Works just like the regular ping command
- Displays fun ASCII art in pink color before ping output
- Maintains all standard ping functionality
- Passes through exit codes from the original ping command

## Installation

```bash
cargo build --release
```

The executable will be in `target/release/bingping`

## Usage

```bash
# Basic usage
bingping example.com

# Specify count (like ping -c)
bingping -c 5 example.com

# Specify interval (like ping -i)
bingping -i 2 example.com

# Specify timeout (like ping -W)
bingping -W 5 example.com

# Disable ASCII art
bingping --no-art example.com
```

## Requirements

- Rust 1.54 or later
- The standard `ping` utility must be installed and in your PATH

## License

MIT