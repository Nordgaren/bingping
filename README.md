# BingPing
I have not reviewed any of the code in this repository. I am not responsible for any damage caused by this software. It was generated entirely by Cursor.

I'll review it later, but for now, the funnies.  


<pre>
                                     ..................................                             
                               .......=+++=+=+++++++++++++++=+++++++++*....                         
                            ....++++++++++++++++++++++++++++++++++++++===+...                       
                         ...++++++++++++++++++++++++++++++++++++*+++++++*+===...                    
                      ...+++++++++++++++++***++*+**++++++++++++++++++++++++++=-..                   
                    ..*++**++*++++++++++++++++++*++++++*++++++++++++**++++++++++=..                 
                  ...*+++++++++++++++*+++++++++++++++++++++**++*++++++++++*++++*#**.                
                 ..++++++++++++++++++=+++++++++++++*+++++++++++++++====++++++++++**+..              
                 **=++++++++++=++++++++=====++++++++++*+++++++++++===+*++++++++==+*+=.              
                .@%+=++++*++======++++++++++=++=+++++=++++++=+====+*###+=+==---===++*.              
                .*+++++++=++=+=-:...:-=+*********+**++++++++++*#******+:...::--+#%#**.              
                .==++#*+++++++*%@@@%=:-+**###*****+***********+*****#*:.=@@@@@@%*+**%.              
                 -*#*++**+*@%#@@@@@@@@@+-=*#####****+*********#####*=-#@@@@@@@+%#=+++.              
                 .#%##****#%#@@%*%###%@@@*+***#####****#####*#####*=#@@@%##%@@@#*-*@@.              
                 .%@#*****+++@@@%%%@@@@@@@@#######*#####*#########+%@@@@@%%@@@@##-+*=.              
                 ..=--=+**+=@@@@@@@@@@@@@+#@**%######*############*%@@@@@@@@@@@+*-##.               
                  .@@@@#**+-*@@@@@@@@@@@@+@@#*#***###**********##*###@@@@@@@@@=#=*%:.               
                   .%%***##*-++#@@@@@@@:-#@@@********+*******#****%@@#+%@@@#.:%*=*#.                
                   .##*****#*+==:....:+@@@%%#+**+************#####***@@%+..=#+:=##-                 
                   .##*******##*****##++-==+*#%%%#*****+**#########*+==-====+*#%#:.                 
                   .*#*#*#######%#************+**************###*+++**##%%%%%%%#:.                  
                    +**##**#######%###**++++++*++++++++++++******+==+===***####:.                   
                    *#**#**######***+*****+++++++++*++++++++***+++++++*+++####:.                    
                   .*****##%#########*****+++++****+++=++=+**++++++*+***#####:.                     
                   .*####*#######%##%###*###**+=+=++*****++++***+++##**##%%#:.                      
                  .*#****#####%###*####****####*++***+++++++++***++****##%%%+                       
                 .#*##**#*###########***********+++=+=====+-===+++******##+*=                       
                .:*******##*##%######**+*****####*#%@@@@%@@@@@@@#########*+=.                       
               .:+****#***###**########****######***@@@@%@@@@@%**####%%%%*++=.                      
               :=*****+++++*#****#*################*=+##@@@@%++##%%#####%##**.                      
              .+**#**###***+++****############*########%@@@#+*##%%%%%%%#%####.                      
             ..*******#%%@%%##**+****###%%%%%%%%%####%##@#*#%%%#%%%%%%%######.                      
           .:+*********#*#%%@@@@@@@@@%#%%%%#%%%%#%%%%%%###%##%%%#%#%%######%#.                      
          ..+*******************#*#*###########%%####%######%#%%%%%%#########.                      
         ::+*##*************#######*##**######%%%#%%%%%#%%%%%@%%%%%#%%%%####*:                      
        .:+**************##***#***##########%##%%%%%%%%%%%%%%%%%#%%%%##%##*#*+.                     
        :**++***+***********#****###*#####%##%%%###%%%%%%%%%####%##%########**.                     
       .+*++++++**************##****###############%##%###%%#%#%%###########**.                     
       .**+++**+*****#********#***#############%####%#%%%%%%#####%%##%####*###.                     
       .#*+**+*++*****##**#**##*####################%#########################:.                    
       .*+*+****#************##***##%##**#########%%###%%#%######%##%##%#%####*-                    
       .*+***#*#**#*******#***##********###*#######*###########%#%#####%#######+.                   
       .#*********#*#************#**********##################%################*.                   
       ...........................................................................                  
</pre>

BingPing is a custom ICMP ping implementation with ASCII art display.

## Features

- Displays Bingus ASCII art before ping output and in replies
- Supports color options for ASCII art:
  - Pink color (default)
  - Rainbow color (with the `--rainbow` flag)
- Custom raw socket ICMP implementation
- Cross-platform support for both Linux and Windows
- Standard ping functionality (count, size, interval, etc.)
- Includes timeout handling and statistics

## Installation

### Linux
```bash
cargo build --release
```

The executable will be in `target/release/bingping`

### Windows
To cross-compile for Windows from Linux:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

The Windows executable will be in `target/x86_64-pc-windows-gnu/release/bingping.exe`

## Usage

```bash
# Basic usage
bingping example.com

# With rainbow colors
bingping --rainbow example.com

# Linux specific options
# ---------------------
# Specify count (number of packets)
bingping -c 5 example.com

# With rainbow colors and specific count
bingping -r -c 5 example.com

# Specify interval between packets in seconds
bingping -i 2 example.com

# Specify wait timeout in seconds
bingping -W 5 example.com

# Specify packet size in bytes
bingping -s 64 example.com

# Specify TTL (Time To Live)
bingping -t 64 example.com

# Windows specific options
# -----------------------
# Specify count (number of packets)
bingping -n 5 example.com

# With rainbow colors (Windows only has long flag option)
bingping --rainbow -n 5 example.com

# Specify wait timeout in milliseconds
bingping -w 1000 example.com

# Specify buffer size in bytes
bingping -l 32 example.com

# Specify TTL (Time To Live)
bingping -i 32 example.com

# Resolve addresses to hostnames
bingping -a example.com
```

## Requirements

- Rust 1.54 or later
- For raw socket functionality: elevated permissions (sudo/administrator)
- For cross-compiling to Windows: MinGW toolchain

## License

