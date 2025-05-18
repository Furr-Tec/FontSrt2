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

### From Source
