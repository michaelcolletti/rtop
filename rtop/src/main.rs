// src/main.rs
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;
use std::time::{Duration, Instant};
use sysinfo::{CpuExt, ProcessExt, System, SystemExt, Pid, Signal};
use termion::raw::IntoRawMode;
use thiserror::Error;
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
    widgets::{Block, Borders, Cell, Gauge, Row, Table, Paragraph},
    Terminal,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Refresh rate in milliseconds
    #[arg(short, long, default_value_t = 250)]
    refresh_rate: u64,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortBy {
    Cpu,
    Memory,
    Name,
    Pid,
}

#[derive(PartialEq)]
enum AppState {
    Main,
    ProcessMenu,
}

struct App {
    system: System,
    selected_process: Option<usize>,
    sort_by: SortBy,
    state: AppState,
}

impl App {
    fn new() -> App {
        App {
            system: System::new_all(),
            selected_process: None,
            sort_by: SortBy::Cpu,
            state: AppState::Main,
        }
    }

    fn update(&mut self) {
        self.system.refresh_all();
    }

    fn get_sorted_processes(&self) -> Vec<(Pid, &sysinfo::Process)> {
        let mut processes: Vec<_> = self.system.processes().iter().map(|(&pid, proc)| (pid, proc)).collect();
        match self.sort_by {
            SortBy::Cpu => processes.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap()),
            SortBy::Memory => processes.sort_by(|a, b| b.1.memory().cmp(&a.1.memory())),
            SortBy::Name => processes.sort_by(|a, b| a.1.name().cmp(b.1.name())),
            SortBy::Pid => processes.sort_by(|a, b| a.0.cmp(&b.0)),
        }
        processes
    }

    fn get_selected_process(&self) -> Option<(Pid, &sysinfo::Process)> {
        self.selected_process.and_then(|idx| self.get_sorted_processes().get(idx).cloned())
    }

    fn send_signal(&mut self, signal: Signal) -> bool {
        if let Some((pid, _)) = self.get_selected_process() {
            if let Some(process) = self.system.process(pid) {
                return process.kill_with(signal).is_some();
            }
        }
        false
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let refresh_rate = Duration::from_millis(args.refresh_rate);

    enable_raw_mode()?;
    let mut stdout = stdout().into_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app, refresh_rate);

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
                    KeyCode::Char('c') => app.sort_by = SortBy::Cpu,
                    KeyCode::Char('m') => app.sort_by = SortBy::Memory,
                    KeyCode::Char('n') => app.sort_by = SortBy::Name,
                    KeyCode::Char('p') => app.sort_by = SortBy::Pid,
                    KeyCode::Char('k') => {
                        if app.state == AppState::Main {
                            app.state = AppState::ProcessMenu;
                        }
                    }
                    KeyCode::Esc => {
                        app.state = AppState::Main;
                    }
                    KeyCode::Char('1') => {
                        if app.state == AppState::ProcessMenu {
                            app.send_signal(Signal::Interrupt);
                            app.state = AppState::Main;
                        }
                    }
                    KeyCode::Char('9') => {
                        if app.state == AppState::ProcessMenu {
                            app.send_signal(Signal::Kill);
                            app.state = AppState::Main;
                        }
                    }
                    KeyCode::Char('2') => {
                        if app.state == AppState::ProcessMenu {
                            app.send_signal(Signal::Quit);
                            app.state = AppState::Main;
                        }
                    }
                    KeyCode::Char('3') => {
                        if app.state == AppState::ProcessMenu {
                            app.send_signal(Signal::Term);
                            app.state = AppState::Main;
                        }
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
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Top gauges
            Constraint::Min(10),    // Process table
            Constraint::Length(1),  // Help text
        ].as_ref())
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

    let processes = app.get_sorted_processes();
    let process_rows: Vec<Row> = processes
        .iter()
        .enumerate()
        .map(|(i, (pid, process))| {
            let selected = app
                .selected_process
                .map_or(false, |selected_index| selected_index == i);
            let style = if selected {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            };
            let cpu_usage = process.cpu_usage();
            let memory_usage = process.memory() as f64 / 1024.0 / 1024.0;
            let virtual_memory_bytes = process.virtual_memory() as f64;
            let virtual_memory = virtual_memory_bytes / 1024.0 / 1024.0 / 1024.0;
            
            let cpu_color = if cpu_usage > 50.0 {
                Color::Red
            } else if cpu_usage > 20.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            
            let mem_color = if memory_usage > 1000.0 {
                Color::Red
            } else if memory_usage > 500.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                Cell::from(pid.to_string()),
                Cell::from(process.name()),
                Cell::from(format!("{:.1}", cpu_usage)).style(Style::default().fg(cpu_color)),
                Cell::from(format!("{:.1} MB", memory_usage)).style(Style::default().fg(mem_color)),
                Cell::from(format!("{:.2} GB", virtual_memory)).style(Style::default().fg(mem_color)),
                Cell::from(format!("{:.1} MB", memory_usage)).style(Style::default().fg(mem_color)),
            ])
            .style(style)
        })
        .collect();

    let process_table = Table::new(process_rows)
        .header(Row::new(vec!["PID", "Name", "CPU%", "RSS", "Virtual", "Private"]))
        .block(Block::default().title("Processes").borders(Borders::ALL))
        .widths(&[
            Constraint::Length(8),    // PID
            Constraint::Min(20),      // Name
            Constraint::Length(8),    // CPU%
            Constraint::Length(12),   // RSS
            Constraint::Length(12),   // Virtual
            Constraint::Length(12),   // Private
        ]);

    let help_text = if app.state == AppState::Main {
        Paragraph::new("Controls: ↑/↓: Select process | c: Sort by CPU | m: Sort by Memory | n: Sort by Name | p: Sort by PID | k: Kill menu | q: Quit")
    } else {
        Paragraph::new("Kill Menu: 1: SIGINT | 2: SIGQUIT | 3: SIGTERM | 9: SIGKILL | ESC: Cancel")
    }
    .style(Style::default().fg(Color::Gray))
    .block(Block::default().borders(Borders::NONE));

    f.render_widget(process_table, chunks[1]);
    f.render_widget(help_text, chunks[2]);

    if app.state == AppState::ProcessMenu {
        let block = Block::default()
            .title("Process Management")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));
        let area = centered_rect(60, 20, f.size());
        f.render_widget(block, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: tui::layout::Rect) -> tui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
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
