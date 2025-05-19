# Font Organizer

A robust Rust tool for organizing font collections based on Foundry and Font Family.

## Features

- **Advanced Font Organization**: Organizes fonts into a structured hierarchy based on foundry and family name
- **Multiple Naming Patterns**: Supports various naming patterns for files and directories:
  - `Family (Subfamily)` - Example: "Helvetica (Bold)"
  - `Foundry Family (Subfamily)` - Example: "Adobe Helvetica (Bold)"
  - `Family Weight` - Example: "Helvetica 700"
  - `Foundry/Family` directory structure
- **Intelligent Font Metadata Extraction**: Pulls comprehensive metadata from font files
- **Foundry Detection**: Automatically detects or infers the font foundry
- **Duplicate Handling**: Smart duplicate detection and management
- **Parallel Processing**: Utilizes multiple cores for faster organization
- **Batch Processing**: Process multiple directories in a single run

## Installation

### Prerequisites
- Rust 1.70.0 or later
- Cargo package manager

### From Source
1. Clone the repository:
```bash
git clone https://github.com/your-username/FontSrt.git
cd FontSrt
```

2. Build the release version:
```bash
cargo build --release
```

3. The binary will be available at `target/release/fontsrt`

## Code Structure

The project has been modularized for better maintainability:

- `main.rs` - Core program flow and entry point
- `error/` - Error handling
- `models/` - Data structures and configuration
- `utils/` - Utility functions (file operations, naming, logging)
- `font/` - Font processing (metadata, foundry detection, weight)
- `organizer/` - Font organization logic
- `cli/` - Command-line interface utilities

## Usage

```bash
FontSrt [OPTIONS] [DIRECTORY]

Options:
    -h, --help                      Show help message
    --debug                         Enable debug logging
    --batch <FILE>                  Process multiple directories
    --foundry-family-subfamily      Use "Foundry Family (Subfamily)" naming
    --family-weight                 Use "Family Weight" naming
    --foundry-family                Use "Foundry/Family" structure
```

### Usage Examples

1. Basic font organization:
```bash
fontsrt /path/to/fonts
```

2. Using Foundry Family naming pattern:
```bash
fontsrt --foundry-family-subfamily /path/to/fonts
```

3. Batch processing multiple directories:
```bash
# Create a batch file (directories.txt):
/path/to/fonts1
/path/to/fonts2
/path/to/fonts3

# Run batch processing:
fontsrt --batch directories.txt
```

4. Debug mode with weight-based naming:
```bash
fontsrt --debug --family-weight /path/to/fonts
```

### Common Workflows

1. Organize by Family:
- Groups fonts by family name
- Maintains subfamily information
- Creates clean, hierarchical structure

2. Group by Foundry:
- First organizes by family
- Then groups families by foundry
- Creates foundry/family hierarchy

3. Batch Processing:
- Process multiple directories
- Consistent organization across all locations
- Automatic foundry detection and grouping

## Module Details

### font/
- `metadata.rs`: Font validation and metadata extraction
- `foundry.rs`: Foundry detection using pattern matching and metadata analysis
- `weight.rs`: Weight determination and style analysis

### utils/
- `file.rs`: File operations with safety checks and error handling
- `naming.rs`: Font name formatting and standardization
- `logging.rs`: Debug logging with configurable output

### organizer/
- `processor.rs`: Core organization logic and font processing
- `batch.rs`: Multi-directory batch processing
- `group.rs`: Foundry-based grouping implementation

### cli/
- `args.rs`: Command-line argument parsing and validation
- `interaction.rs`: User interaction and input handling

## Technical Details

### Dependencies
- `font-kit 0.14.2`: Font metadata extraction and manipulation
- `ttf-parser 0.24.1`: Low-level font file validation
- `rayon 1.5`: Parallel processing for performance
- `regex 1`: Pattern matching for foundry detection
- `lazy_static 1.4`: Efficient static pattern compilation

### Build Configuration

Release profile is optimized for performance:
```toml
[profile.release]
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
opt-level = 3          # Maximum optimization
panic = "abort"        # Smaller binary size
```

Development profile for debugging:
```toml
[profile.dev]
opt-level = 0          # Fast compilation
debug = true           # Debug symbols
```

## Contributing

When contributing to this project, please ensure:
1. All code follows the established modular structure
2. New functionality is properly documented
3. Existing tests pass and new tests are added for new features
4. Code follows Rust best practices and formatting guidelines

## License

This project is licensed under the MIT License - see the LICENSE file for details.
