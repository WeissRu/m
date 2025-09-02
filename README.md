# m - File Fast Mover

A CLI tool for moving newly created files (like those in download directories) to the current directory.

## Compilation

### Prerequisites

- Rust 1.70+ (for edition 2024 support)

### Build

```bash
cargo build --release
```

The compiled binary will be located at `target/release/m`

## Usage

Run in any directory:

```bash
./m
```

The program will:
1. Read configuration from `~/.config/m/m.json`
2. Scan configured directories for recently created files
3. If files are found, display an interactive list for selection
4. Move the selected file to the current directory

### Usage Example

```
? Select a file to move:  
> 14:43 12KB      document.docx
  14:41 3MB       hello.png
[Use arrow keys to navigate, press Enter to select]
```

After pressing Enter:

```
Successfully moved 'document.docx' to current directory
```

## Configuration

The configuration file is located at `~/.config/m/m.json` and contains:

- **`source_dir`**: Array of directory paths to monitor
- **`time_limit`**: File creation time limit in minutes

### Default Configuration

A default configuration file will be created on first run:

```json
{
  "source_dir": [
    "/home/user/Downloads/"
  ],
  "time_limit": 20
}
```

### Configuration Example

```json
{
  "source_dir": [
    "/home/user/Downloads/",
    "/home/user/Desktop/",
    "/tmp/"
  ],
  "time_limit": 30
}
```