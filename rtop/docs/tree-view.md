# Tree View Feature Implementation

## Overview
Add hierarchical process tree view similar to htop, showing parent-child relationships between processes.

## Required Changes to main.rs

### 1. Add Process Tree Structure
```rust
#[derive(Debug)]
struct ProcessNode {
    pid: Pid,
    process: sysinfo::Process,
    children: Vec<ProcessNode>,
    expanded: bool,
}

struct App {
    // ... existing fields ...
    process_tree: Vec<ProcessNode>,
    show_tree: bool,
}
```

### 2. Add Tree Building Function
```rust
impl App {
    fn build_process_tree(&mut self) {
        let mut root_processes = Vec::new();
        let processes = self.system.processes();
        
        // First pass: Create all nodes
        let mut nodes: HashMap<Pid, ProcessNode> = HashMap::new();
        for (&pid, process) in processes {
            nodes.insert(pid, ProcessNode {
                pid,
                process: process.clone(),
                children: Vec::new(),
                expanded: true,
            });
        }
        
        // Second pass: Build tree structure
        for (&pid, process) in processes {
            if let Some(ppid) = process.parent() {
                if let Some(parent_node) = nodes.get_mut(&ppid) {
                    if let Some(child_node) = nodes.remove(&pid) {
                        parent_node.children.push(child_node);
                    }
                }
            } else {
                if let Some(root_node) = nodes.remove(&pid) {
                    root_processes.push(root_node);
                }
            }
        }
        
        self.process_tree = root_processes;
    }
}
```

### 3. Add Tree View Rendering
```rust
fn render_process_tree<B: Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    tree: &[ProcessNode],
    selected: Option<usize>,
    depth: usize,
) {
    for (i, node) in tree.iter().enumerate() {
        let is_selected = selected == Some(i);
        let style = if is_selected {
            Style::default().bg(Color::Blue)
        } else {
            Style::default()
        };
        
        // Render process line with indentation
        let indent = "  ".repeat(depth);
        let expand_symbol = if !node.children.is_empty() {
            if node.expanded { "▼" } else { "▶" }
        } else {
            " "
        };
        
        let row = Row::new(vec![
            Cell::from(format!("{}{} {}", indent, expand_symbol, node.pid)),
            Cell::from(node.process.name()),
            // ... other columns ...
        ]).style(style);
        
        f.render_widget(row, area);
        
        // Recursively render children if expanded
        if node.expanded {
            render_process_tree(f, area, &node.children, selected, depth + 1);
        }
    }
}
```

### 4. Add Tree Navigation Controls
```rust
// In run_app function, add:
match key.code {
    // ... existing key handlers ...
    KeyCode::Right => {
        if let Some(idx) = app.selected_process {
            if let Some(node) = get_node_at_index(&app.process_tree, idx) {
                node.expanded = true;
            }
        }
    }
    KeyCode::Left => {
        if let Some(idx) = app.selected_process {
            if let Some(node) = get_node_at_index(&app.process_tree, idx) {
                node.expanded = false;
            }
        }
    }
    KeyCode::Char('t') => {
        app.show_tree = !app.show_tree;
    }
}
```

### 5. Update UI Function
```rust
fn ui<B: Backend>(f: &mut tui::Frame<B>, app: &mut App) {
    // ... existing layout code ...
    
    if app.show_tree {
        render_process_tree(f, chunks[1], &app.process_tree, app.selected_process, 0);
    } else {
        // Render flat process list
        render_process_list(f, chunks[1], app);
    }
}
```

## New Dependencies
```toml
[dependencies]
# ... existing dependencies ...
hashbrown = "0.13"  # For HashMap
```

## Testing Plan
1. Test tree building with various process hierarchies
2. Test expansion/collapse functionality
3. Test navigation in tree view
4. Test switching between tree and list views
5. Test performance with large process trees

## Migration Steps
1. Add new data structures
2. Implement tree building
3. Add tree rendering
4. Add navigation controls
5. Add view switching
6. Test and optimize performance 