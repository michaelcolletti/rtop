# RTop for Mac and Windows

A Rust-based system monitor inspired by htop and top, designed to run on macOS and Windows.

## Features

- CPU and memory usage display.
- Process list with CPU and memory consumption.
- Interactive process selection.
- Cross-platform support (macOS and Windows coming soon).

## Installation

1. Clone the repository.
2. Install Rust and Cargo.
3. Run `cargo build --release`.
4. Run the executable from `target/release/`.

## Usage

- Run the executable.
- Use the arrow keys to navigate the process list.
- Press `q` to quit.

## Improvements

- Add support for sorting processes by CPU or memory usage.
- Implement process killing.
- Display network usage and disk I/O.
- Improve UI with more detailed information.
- Adding color coding for resource usage.
- Implement a config file for customization.
- Add support for filtering processes.

## Architecture and Process

1.  **Dependencies:**
    * `sysinfo` for system information.
    * `termion` or `crossterm` for terminal manipulation.
    * `tui` for terminal UI rendering.
    * `chrono` for time related functions.
2.  **System Information Retrieval:**
    * `sysinfo` is used to gather CPU, memory, and process information.
3.  **Terminal UI:**
    * `tui` is used to create a responsive and interactive terminal interface.
4.  **Event Handling:**
    * `crossterm` is used to handle keyboard input.
5.  **Rendering:**
    * The `ui` function renders the system information and process list.
6.  **Update Loop:**
    * The `run_app` function updates the system information at a specified tick rate.
7.  **CI/CD:**
    * GitHub Actions is used for automated building, testing, and deployment.
    * The workflow builds the project for macOS and Windows.
    * Artifacts are uploaded for download.

## Testing

The CI/CD pipeline runs `cargo test` to ensure code quality. Add unit and integration tests to increase coverage.
