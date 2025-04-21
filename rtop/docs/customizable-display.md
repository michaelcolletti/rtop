# Customizable Display Feature Implementation

## Overview
Add the ability to customize which columns are displayed and their order, with persistence of preferences.

## Required Changes to main.rs

### 1. Add Column Configuration Structure
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Column {
    Pid,
    Name,
    Cpu,
    Memory,
    Virtual,
    Private,
    User,
    State,
    Priority,
    StartTime,
    CpuTime,
}

#[derive(Debug, Clone)]
struct ColumnConfig {
    column: Column,
    visible: bool,
    width: Constraint,
    order: usize,
}

struct App {
    // ... existing fields ...
    column_configs: Vec<ColumnConfig>,
    config_path: PathBuf,
}
```

### 2. Add Configuration Management
```rust
impl App {
    fn load_config(&mut self) {
        if let Ok(config_file) = std::fs::read_to_string(&self.config_path) {
            if let Ok(configs) = serde_json::from_str::<Vec<ColumnConfig>>(&config_file) {
                self.column_configs = configs;
                return;
            }
        }
        
        // Default configuration
        self.column_configs = vec![
            ColumnConfig {
                column: Column::Pid,
                visible: true,
                width: Constraint::Length(8),
                order: 0,
            },
            ColumnConfig {
                column: Column::Name,
                visible: true,
                width: Constraint::Min(20),
                order: 1,
            },
            // ... other default columns ...
        ];
    }

    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = serde_json::to_string_pretty(&self.column_configs)?;
        std::fs::write(&self.config_path, config_str)?;
        Ok(())
    }

    fn get_visible_columns(&self) -> Vec<&ColumnConfig> {
        let mut visible: Vec<_> = self.column_configs
            .iter()
            .filter(|c| c.visible)
            .collect();
        visible.sort_by_key(|c| c.order);
        visible
    }
}
```

### 3. Add Column Management UI
```rust
fn render_column_menu<B: Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    configs: &[ColumnConfig],
    selected: Option<usize>,
) {
    let block = Block::default()
        .title("Column Configuration")
        .borders(Borders::ALL);
    f.render_widget(block, area);

    let list_items: Vec<ListItem> = configs
        .iter()
        .enumerate()
        .map(|(i, config)| {
            let style = if Some(i) == selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(format!(
                "[{}] {} - Width: {:?} - Order: {}",
                if config.visible { "✓" } else { " " },
                format!("{:?}", config.column),
                config.width,
                config.order
            ))
            .style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default().bg(Color::Blue));
    f.render_stateful_widget(list, area, &mut ListState::with_selected(selected));
}
```

### 4. Add Column Management Controls
```rust
// In run_app function, add:
match key.code {
    // ... existing key handlers ...
    KeyCode::Char('c') => {
        app.state = AppState::ColumnConfig;
    }
    KeyCode::Char(' ') => {
        if let Some(idx) = app.selected_column {
            app.column_configs[idx].visible = !app.column_configs[idx].visible;
        }
    }
    KeyCode::Char('+') => {
        if let Some(idx) = app.selected_column {
            app.column_configs[idx].order = app.column_configs[idx].order.saturating_sub(1);
        }
    }
    KeyCode::Char('-') => {
        if let Some(idx) = app.selected_column {
            app.column_configs[idx].order = app.column_configs[idx].order.saturating_add(1);
        }
    }
    KeyCode::Char('w') => {
        if let Some(idx) = app.selected_column {
            app.column_configs[idx].width = match app.column_configs[idx].width {
                Constraint::Length(n) => Constraint::Length(n + 1),
                _ => Constraint::Length(8),
            };
        }
    }
    KeyCode::Char('s') => {
        if let Some(idx) = app.selected_column {
            app.column_configs[idx].width = match app.column_configs[idx].width {
                Constraint::Length(n) if n > 1 => Constraint::Length(n - 1),
                _ => Constraint::Length(8),
            };
        }
    }
}
```

### 5. Update Process Table Rendering
```rust
fn render_process_table<B: Backend>(f: &mut tui::Frame<B>, app: &mut App, area: tui::layout::Rect) {
    let visible_columns = app.get_visible_columns();
    let widths: Vec<_> = visible_columns.iter().map(|c| c.width).collect();
    
    let header_cells: Vec<Cell> = visible_columns
        .iter()
        .map(|c| Cell::from(format!("{:?}", c.column)))
        .collect();
    
    let header = Row::new(header_cells);
    
    let process_rows: Vec<Row> = app.get_sorted_processes()
        .iter()
        .enumerate()
        .map(|(i, (pid, process))| {
            let selected = app.selected_process == Some(i);
            let style = if selected {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            };
            
            let cells: Vec<Cell> = visible_columns
                .iter()
                .map(|c| match c.column {
                    Column::Pid => Cell::from(pid.to_string()),
                    Column::Name => Cell::from(process.name()),
                    Column::Cpu => Cell::from(format!("{:.1}", process.cpu_usage())),
                    // ... other column cases ...
                })
                .collect();
            
            Row::new(cells).style(style)
        })
        .collect();
    
    let table = Table::new(process_rows)
        .header(header)
        .block(Block::default().title("Processes").borders(Borders::ALL))
        .widths(&widths);
    
    f.render_widget(table, area);
}
```

## New Dependencies
```toml
[dependencies]
# ... existing dependencies ...
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Testing Plan
1. Test configuration loading/saving
2. Test column visibility toggling
3. Test column reordering
4. Test width adjustment
5. Test persistence across sessions
6. Test performance with many columns

## Migration Steps
1. Add column configuration structures
2. Implement configuration management
3. Add column management UI
4. Update process table rendering
5. Add persistence
6. Test and optimize performance 