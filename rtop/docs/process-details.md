# Process Details Feature Implementation

## Overview
Add a detailed process information panel that shows when a process is selected, displaying additional process metrics and information.

## Required Changes to main.rs

### 1. Add Process Details Structure
```rust
struct ProcessDetails {
    pid: Pid,
    name: String,
    state: String,
    user: String,
    priority: i32,
    start_time: String,
    cpu_time: String,
    memory_details: MemoryDetails,
    io_stats: IoStats,
}

struct MemoryDetails {
    resident: u64,
    shared: u64,
    text: u64,
    data: u64,
}

struct IoStats {
    read_bytes: u64,
    write_bytes: u64,
    read_ops: u64,
    write_ops: u64,
}

struct App {
    // ... existing fields ...
    show_details: bool,
    details_cache: Option<ProcessDetails>,
}
```

### 2. Add Details Collection Function
```rust
impl App {
    fn collect_process_details(&mut self, pid: Pid) -> Option<ProcessDetails> {
        if let Some(process) = self.system.process(pid) {
            Some(ProcessDetails {
                pid,
                name: process.name().to_string(),
                state: process.status().to_string(),
                user: process.user_id().to_string(),
                priority: process.nice(),
                start_time: format_time(process.start_time()),
                cpu_time: format_duration(process.cpu_time()),
                memory_details: MemoryDetails {
                    resident: process.memory(),
                    shared: process.shared_memory(),
                    text: process.virtual_memory(),
                    data: process.data_memory(),
                },
                io_stats: IoStats {
                    read_bytes: process.disk_usage().total_read_bytes,
                    write_bytes: process.disk_usage().total_written_bytes,
                    read_ops: process.disk_usage().read_ops,
                    write_ops: process.disk_usage().write_ops,
                },
            })
        } else {
            None
        }
    }
}
```

### 3. Add Details Panel Rendering
```rust
fn render_details_panel<B: Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    details: &ProcessDetails,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Basic Info
            Constraint::Length(3),  // Memory Info
            Constraint::Length(3),  // IO Info
            Constraint::Min(0),     // Spacer
        ])
        .split(area);

    // Header
    let header = Block::default()
        .title(format!("Process Details - PID: {}", details.pid))
        .borders(Borders::ALL);
    f.render_widget(header, chunks[0]);

    // Basic Info
    let basic_info = Paragraph::new(vec![
        Spans::from(vec![
            Span::raw(format!("Name: {} | State: {} | User: {}", 
                details.name, details.state, details.user)),
        ]),
        Spans::from(vec![
            Span::raw(format!("Priority: {} | Start Time: {} | CPU Time: {}", 
                details.priority, details.start_time, details.cpu_time)),
        ]),
    ]);
    f.render_widget(basic_info, chunks[1]);

    // Memory Info
    let memory_info = Paragraph::new(vec![
        Spans::from(vec![
            Span::raw(format!("Resident: {} MB | Shared: {} MB", 
                details.memory_details.resident / 1024 / 1024,
                details.memory_details.shared / 1024 / 1024)),
        ]),
        Spans::from(vec![
            Span::raw(format!("Text: {} MB | Data: {} MB", 
                details.memory_details.text / 1024 / 1024,
                details.memory_details.data / 1024 / 1024)),
        ]),
    ]);
    f.render_widget(memory_info, chunks[2]);

    // IO Info
    let io_info = Paragraph::new(vec![
        Spans::from(vec![
            Span::raw(format!("Read: {} MB | Write: {} MB", 
                details.io_stats.read_bytes / 1024 / 1024,
                details.io_stats.write_bytes / 1024 / 1024)),
        ]),
        Spans::from(vec![
            Span::raw(format!("Read Ops: {} | Write Ops: {}", 
                details.io_stats.read_ops,
                details.io_stats.write_ops)),
        ]),
    ]);
    f.render_widget(io_info, chunks[3]);
}
```

### 4. Add Details Toggle Control
```rust
// In run_app function, add:
match key.code {
    // ... existing key handlers ...
    KeyCode::Char('d') => {
        app.show_details = !app.show_details;
        if app.show_details {
            if let Some(pid) = app.get_selected_process().map(|(p, _)| p) {
                app.details_cache = app.collect_process_details(pid);
            }
        }
    }
}
```

### 5. Update UI Function
```rust
fn ui<B: Backend>(f: &mut tui::Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Top gauges
            if app.show_details {
                Constraint::Percentage(60)  // Process list
            } else {
                Constraint::Min(10)
            },
            if app.show_details {
                Constraint::Percentage(40)  // Details panel
            } else {
                Constraint::Length(0)
            },
            Constraint::Length(1),  // Help text
        ])
        .split(f.size());

    // ... existing gauge rendering ...

    if app.show_details {
        if let Some(ref details) = app.details_cache {
            render_details_panel(f, chunks[2], details);
        }
    }

    // ... rest of the UI code ...
}
```

## New Dependencies
```toml
[dependencies]
# ... existing dependencies ...
chrono = "0.4"  # For time formatting
```

## Testing Plan
1. Test details collection for various process types
2. Test panel rendering and layout
3. Test details update on process selection
4. Test panel toggle functionality
5. Test performance impact

## Migration Steps
1. Add new data structures
2. Implement details collection
3. Add details panel rendering
4. Add panel toggle controls
5. Update UI layout
6. Test and optimize performance 