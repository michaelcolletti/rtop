// src/main.rs
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;
use std::time::{Duration, Instant};
use sysinfo::{CpuExt, ProcessExt, System, SystemExt};
use termion::raw::IntoRawMode;
/// # Terminal UI Components
///
/// This module imports the necessary components from the `tui` crate to create a terminal user interface.
///
/// ## Imports
///
/// - `backend`: Terminal backends, specifically `CrosstermBackend` for cross-platform terminal handling
/// - `layout`: Layout components for arranging UI elements in different constraints and directions
/// - `style`: Styling options for UI elements, including colors and text modifications
/// - `text`: Text components for displaying and formatting text in the UI
/// - `widgets`: UI widgets including:
///   - `Block`: Basic rectangular UI element that can have borders
///   - `Borders`: Border styles for blocks
///   - `Gauge`: Progress bar or meter visualization
///   - `List` and `ListItem`: Components for displaying lists of items
///   - `Paragraph`: Text display with various formatting options
///   - `Table`, `Row`, `Cell`: Table display components
/// - `Terminal`: Main terminal handling component that manages the UI
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Gauge, Row, Table},
    Terminal,
};

struct App {
    system: System,
    selected_process: Option<usize>,
}

impl App {
    fn new() -> App {
        App {
            system: System::new_all(),
            selected_process: None,
        }
    }
    fn update(&mut self) {
        self.system.refresh_all();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout().into_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        if app.selected_process.is_none() {
                            app.selected_process = Some(0);
                        } else {
                            app.selected_process = app.selected_process.map(|i| {
                                let process_count = app.system.processes().len();
                                if i < process_count - 1 {
                                    i + 1
                                } else {
                                    i
                                }
                            });
                        }
                    }
                    KeyCode::Up => {
                        app.selected_process =
                            app.selected_process.map(|i| if i > 0 { i - 1 } else { 0 });
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut tui::Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.size());

    let cpu_usage = app.system.global_cpu_info().cpu_usage();
    let mem_usage = app.system.used_memory() as f64 / app.system.total_memory() as f64;

    let cpu_gauge = Gauge::default()
        .block(Block::default().title("CPU Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(cpu_usage.round() as u16);

    let mem_gauge = Gauge::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent((mem_usage * 100.0).round() as u16);

    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    f.render_widget(cpu_gauge, top_layout[0]);
    f.render_widget(mem_gauge, top_layout[1]);

    let processes: Vec<_> = app.system.processes().iter().collect();
    let process_rows: Vec<Row> = processes
        .iter()
        .enumerate()
        .map(|(i, (&pid, process))| {
            let selected = app
                .selected_process
                .map_or(false, |selected_index| selected_index == i);
            let style = if selected {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(pid.to_string()),
                Cell::from(process.name()),
                Cell::from(format!("{:.1}", process.cpu_usage())),
                Cell::from(format!("{:.1}", process.memory() as f64 / 1024.0 / 1024.0)),
                Cell::from(format!("{:.1}", process.memory() as f64 / 1024.0 / 1024.0)),
            ])
            .style(style)
        })
        .collect();

    let process_table = Table::new(process_rows)
        .header(Row::new(vec!["PID", "Name", "CPU%", "Mem%"]))
        .block(Block::default().title("Processes").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ]);

    f.render_widget(process_table, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app_update() {
        let mut app = App::new();
        let initial_process_count = app.system.processes().len();
        app.update();
        assert!(app.system.processes().len() >= initial_process_count);
    }
}