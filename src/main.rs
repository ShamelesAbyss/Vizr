use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline},
    Terminal,
};
use std::{
    collections::VecDeque,
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone)]
enum InputMode {
    Synthetic,
    Stdin,
    File(String),
    Tail(String),
    Ping(String),
    Build,
}

struct AppState {
    tick: usize,
    signal: Vec<u64>,
    pulse: Vec<u64>,
    noise: Vec<u64>,
    events: VecDeque<String>,
    energy: u16,
    stability: u16,
    anomaly: u16,
    sync_rate: u16,
    mode_label: String,
    last_event: Instant,
    rng: SimpleRng,
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);

        self.state
    }

    fn range(&mut self, start: u64, end: u64) -> u64 {
        if end <= start {
            return start;
        }

        start + (self.next_u64() % (end - start))
    }
}

fn main() -> Result<()> {
    let mode = parse_mode();
    let receiver = start_input_thread(mode.clone());

    let seed = Local::now().timestamp_nanos_opt().unwrap_or_default() as u64;

    let mut state = AppState {
        tick: 0,
        signal: vec![0; 90],
        pulse: vec![0; 90],
        noise: vec![0; 90],
        events: VecDeque::new(),
        energy: 0,
        stability: 100,
        anomaly: 0,
        sync_rate: 0,
        mode_label: mode_label(&mode),
        last_event: Instant::now(),
        rng: SimpleRng::new(seed),
    };

    push_event(&mut state, "VIZR engine initialized");
    push_event(&mut state, "adaptive render matrix online");
    push_event(&mut state, "local telemetry clock locked");
    push_event(&mut state, "named adapters: ping HOST | build | file PATH | tail PATH | stdin");

    run_ui(&mut state, receiver)
}

fn parse_mode() -> InputMode {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--stdin") | Some("stdin") => InputMode::Stdin,
        Some("--file") | Some("file") => {
            InputMode::File(args.get(2).cloned().unwrap_or_else(|| "Cargo.toml".to_string()))
        }
        Some("tail") => {
            InputMode::Tail(args.get(2).cloned().unwrap_or_else(|| "Cargo.toml".to_string()))
        }
        Some("ping") => {
            InputMode::Ping(args.get(2).cloned().unwrap_or_else(|| "google.com".to_string()))
        }
        Some("build") => InputMode::Build,
        _ => InputMode::Synthetic,
    }
}

fn mode_label(mode: &InputMode) -> String {
    match mode {
        InputMode::Synthetic => "SYNTHETIC STREAM".to_string(),
        InputMode::Stdin => "STDIN STREAM".to_string(),
        InputMode::File(path) => format!("FILE: {}", path),
        InputMode::Tail(path) => format!("TAIL: {}", path),
        InputMode::Ping(host) => format!("PING: {}", host),
        InputMode::Build => "CARGO BUILD".to_string(),
    }
}

fn start_input_thread(mode: InputMode) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();

    match mode {
        InputMode::Synthetic => {}

        InputMode::Stdin => {
            thread::spawn(move || {
                let stdin = io::stdin();

                for line in stdin.lock().lines().map_while(std::result::Result::ok) {
                    let _ = tx.send(line);
                }
            });
        }

        InputMode::File(path) => {
            thread::spawn(move || {
                match File::open(&path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);

                        for line in reader.lines().map_while(std::result::Result::ok) {
                            let _ = tx.send(line);
                            thread::sleep(Duration::from_millis(80));
                        }
                    }
                    Err(_) => {
                        let _ = tx.send(format!("ERROR unable to open file: {}", path));
                    }
                }
            });
        }

        InputMode::Tail(path) => {
            thread::spawn(move || loop {
                match File::open(&path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);

                        for line in reader.lines().map_while(std::result::Result::ok) {
                            let _ = tx.send(line);
                            thread::sleep(Duration::from_millis(90));
                        }
                    }
                    Err(_) => {
                        let _ = tx.send(format!("ERROR unable to tail file: {}", path));
                        thread::sleep(Duration::from_secs(2));
                    }
                }

                thread::sleep(Duration::from_millis(500));
            });
        }

        InputMode::Ping(host) => {
            thread::spawn(move || {
                let mut child = Command::new("ping")
                    .arg(&host)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn();

                match child.as_mut() {
                    Ok(child) => {
                        if let Some(stdout) = child.stdout.take() {
                            let reader = BufReader::new(stdout);

                            for line in reader.lines().map_while(std::result::Result::ok) {
                                let _ = tx.send(line);
                            }
                        }
                    }
                    Err(_) => {
                        let _ = tx.send(format!("ERROR unable to start ping {}", host));
                    }
                }
            });
        }

        InputMode::Build => {
            thread::spawn(move || {
                let mut child = Command::new("cargo")
                    .arg("build")
                    .arg("--release")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn();

                match child.as_mut() {
                    Ok(child) => {
                        if let Some(stderr) = child.stderr.take() {
                            let reader = BufReader::new(stderr);

                            for line in reader.lines().map_while(std::result::Result::ok) {
                                let _ = tx.send(line);
                            }
                        }

                        let _ = child.wait();
                        let _ = tx.send("FINISHED build adapter completed".to_string());
                    }
                    Err(_) => {
                        let _ = tx.send("ERROR unable to start cargo build".to_string());
                    }
                }
            });
        }
    }

    rx
}

fn run_ui(state: &mut AppState, receiver: Receiver<String>) -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        update_state(state, &receiver);

        terminal.draw(|f| {
            let area = f.area();

            let root = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Length(7),
                    Constraint::Min(12),
                    Constraint::Length(3),
                ])
                .split(area);

            let spinner = ["◢", "◣", "◤", "◥", "◆", "◇"][state.tick % 6];
            let glitch = ["░▒▓", "▒▓█", "▓█▓", "█▓▒", "▓▒░", "▒░▒"][state.tick % 6];

            let header = Paragraph::new(vec![
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        spinner,
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " VIZR ",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        ":: UNIVERSAL TELEMETRY VISUALIZER ",
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(glitch.repeat(12), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled("  adapter: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        state.mode_label.clone(),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  |  local: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(local_time_short(), Style::default().fg(Color::Green)),
                    Span::styled("  |  q quit  |  m mode", Style::default().fg(Color::DarkGray)),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).title(" ABYSS DISPLAY BUS "));

            f.render_widget(header, root[0]);

            let meters = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(root[1]);

            f.render_widget(metric("ENERGY", state.energy, Color::Cyan), meters[0]);
            f.render_widget(metric("STABILITY", state.stability, Color::Green), meters[1]);
            f.render_widget(metric("ANOMALY", state.anomaly, Color::Red), meters[2]);
            f.render_widget(metric("SYNC RATE", state.sync_rate, Color::Magenta), meters[3]);

            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(42),
                    Constraint::Percentage(33),
                    Constraint::Percentage(25),
                ])
                .split(root[2]);

            let left = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ])
                .split(body[0]);

            f.render_widget(
                Sparkline::default()
                    .block(Block::default().borders(Borders::ALL).title(" SIGNAL WAVE "))
                    .data(&state.signal)
                    .style(Style::default().fg(Color::Cyan))
                    .bar_set(symbols::bar::NINE_LEVELS),
                left[0],
            );

            f.render_widget(
                Sparkline::default()
                    .block(Block::default().borders(Borders::ALL).title(" SYNC / PULSE FIELD "))
                    .data(&state.pulse)
                    .style(Style::default().fg(Color::Green))
                    .bar_set(symbols::bar::NINE_LEVELS),
                left[1],
            );

            f.render_widget(
                Sparkline::default()
                    .block(Block::default().borders(Borders::ALL).title(" NOISE / DISTORTION "))
                    .data(&state.noise)
                    .style(Style::default().fg(Color::Red))
                    .bar_set(symbols::bar::NINE_LEVELS),
                left[2],
            );

            let matrix_width = body[1].width.saturating_sub(2) as usize;
            let matrix_height = body[1].height.saturating_sub(2) as usize;

            f.render_widget(
                Paragraph::new(generate_matrix(
                    state.tick,
                    matrix_width,
                    matrix_height,
                    state.energy,
                    state.anomaly,
                    state.sync_rate,
                ))
                .block(Block::default().borders(Borders::ALL).title(" ADAPTIVE RENDER MATRIX "))
                .style(Style::default().fg(matrix_color(state.anomaly))),
                body[1],
            );

            let events: Vec<ListItem> = state
                .events
                .iter()
                .rev()
                .take(18)
                .map(|e| {
                    let color = if e.contains("ERROR") || e.contains("FAIL") || e.contains("PANIC") {
                        Color::Red
                    } else if e.contains("WARN") || e.contains("WARNING") || e.contains("MODE") {
                        Color::Yellow
                    } else if e.contains("FINISHED") || e.contains("SUCCESS") || e.contains("OK") {
                        Color::Green
                    } else {
                        Color::Cyan
                    };

                    ListItem::new(Line::from(Span::styled(e.clone(), Style::default().fg(color))))
                })
                .collect();

            f.render_widget(
                List::new(events)
                    .block(Block::default().borders(Borders::ALL).title(" EVENT FEED LOCAL TIME ")),
                body[2],
            );

            let footer = Paragraph::new(Line::from(vec![
                Span::styled(
                    " STATUS ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " adaptive matrix + local timestamps + named adapters active. ",
                    Style::default().fg(Color::Gray),
                ),
            ]))
            .block(Block::default().borders(Borders::ALL));

            f.render_widget(footer, root[3]);
        })?;

        if event::poll(Duration::from_millis(90))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('m') => cycle_mode(state),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn update_state(state: &mut AppState, receiver: &Receiver<String>) {
    state.tick = state.tick.wrapping_add(1);

    let mut consumed_input = false;

    while let Ok(line) = receiver.try_recv() {
        consumed_input = true;
        ingest_line(state, &line);
    }

    if !consumed_input && is_synthetic_mode(&state.mode_label) {
        synthetic_tick(state);
    } else if !consumed_input {
        idle_tick(state);
    }
}

fn ingest_line(state: &mut AppState, line: &str) {
    let clean = line.trim();

    if clean.is_empty() {
        return;
    }

    let upper = clean.to_uppercase();

    let mut energy = (clean.len() as u64).min(100);
    let mut anomaly = 8u64;
    let mut sync = 70u64;

    if state.mode_label.starts_with("PING") {
        if let Some(ms) = extract_ping_ms(clean) {
            energy = ms.min(100);
            anomaly = if ms < 50 {
                5
            } else if ms < 120 {
                25
            } else {
                75
            };
            sync = 100u64.saturating_sub((ms / 2).min(95));
        } else if upper.contains("UNREACHABLE")
            || upper.contains("TIMEOUT")
            || upper.contains("FAIL")
            || upper.contains("LOSS")
        {
            energy = 95;
            anomaly = 100;
            sync = 0;
        }
    } else if state.mode_label == "CARGO BUILD" {
        if upper.contains("ERROR") || upper.contains("FAILED") {
            energy = 95;
            anomaly = 100;
            sync = 15;
        } else if upper.contains("WARNING") || upper.contains("WARN") {
            energy = 70;
            anomaly = 55;
            sync = 50;
        } else if upper.contains("COMPILING") {
            energy = 75;
            anomaly = 15;
            sync = 80;
        } else if upper.contains("FINISHED") {
            energy = 35;
            anomaly = 2;
            sync = 100;
        }
    } else if state.mode_label.starts_with("FILE") || state.mode_label.starts_with("TAIL") {
        energy = (clean.len() as u64).min(100);
        anomaly = if upper.contains("ERROR") || upper.contains("FAIL") || upper.contains("PANIC") {
            90
        } else if upper.contains("WARN") || upper.contains("WARNING") {
            55
        } else {
            10
        };
        sync = 85;
    } else if upper.contains("ERROR") || upper.contains("FAIL") || upper.contains("PANIC") {
        anomaly = 95;
        energy = 90;
        sync = 20;
    } else if upper.contains("WARN") || upper.contains("WARNING") {
        anomaly = 60;
        energy = 65;
        sync = 55;
    } else if upper.contains("FINISHED") || upper.contains("SUCCESS") || upper.contains("OK") {
        anomaly = 5;
        energy = 35;
        sync = 100;
    }

    apply_metrics(state, energy, anomaly, sync);
    push_event(state, clean);
}

fn extract_ping_ms(line: &str) -> Option<u64> {
    let marker = "time=";
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];

    let value = rest
        .split_whitespace()
        .next()?
        .trim_end_matches("ms")
        .parse::<f64>()
        .ok()?;

    Some(value.round() as u64)
}

fn synthetic_tick(state: &mut AppState) {
    let wave = (((state.tick as f64 / 5.0).sin() + 1.0) * 40.0) as u64;

    let pulse = if state.tick % 16 < 4 {
        90
    } else {
        state.rng.range(8, 35)
    };

    let noise = match state.mode_label.as_str() {
        "CALM STREAM" => state.rng.range(0, 20),
        "CHAOS STREAM" => state.rng.range(20, 100),
        "PULSE STORM" => {
            if state.tick % 10 == 0 {
                100
            } else {
                state.rng.range(5, 55)
            }
        }
        _ => state.rng.range(5, 70),
    };

    let energy = ((wave + pulse + noise) / 3).min(100);
    apply_metrics(state, energy, noise.min(100), pulse.min(100));

    if state.last_event.elapsed() > Duration::from_millis(900) {
        let roll = state.rng.range(0, 100);

        if roll > 85 {
            push_event(state, "ANOMALY spike detected in render stream");
        } else if roll > 70 {
            push_event(state, "signal packet normalized");
        } else if roll > 55 {
            push_event(state, "frame buffer pulse synchronized");
        } else {
            push_event(state, "visual telemetry updated");
        }

        state.last_event = Instant::now();
    }
}

fn idle_tick(state: &mut AppState) {
    apply_metrics(
        state,
        state.energy.saturating_sub(1) as u64,
        state.anomaly.saturating_sub(1) as u64,
        state.sync_rate.saturating_sub(1) as u64,
    );
}

fn apply_metrics(state: &mut AppState, energy: u64, anomaly: u64, sync: u64) {
    let energy = energy.min(100);
    let anomaly = anomaly.min(100);
    let sync = sync.min(100);

    push_graph(&mut state.signal, energy);
    push_graph(&mut state.pulse, sync);
    push_graph(&mut state.noise, anomaly);

    state.energy = energy as u16;
    state.anomaly = anomaly as u16;
    state.sync_rate = sync as u16;

    let stability =
        100i32 - ((state.anomaly as f32 * 0.7) as i32) + ((state.sync_rate as f32 * 0.3) as i32);

    state.stability = stability.clamp(0, 100) as u16;
}

fn is_synthetic_mode(mode: &str) -> bool {
    matches!(
        mode,
        "SYNTHETIC STREAM" | "CALM STREAM" | "CHAOS STREAM" | "PULSE STORM"
    )
}

fn push_graph(graph: &mut Vec<u64>, value: u64) {
    graph.push(value.min(100));

    if graph.len() > 90 {
        graph.remove(0);
    }
}

fn push_event(state: &mut AppState, msg: &str) {
    state.events.push_back(format!("[{}] {}", local_time_full(), msg));

    if state.events.len() > 300 {
        state.events.pop_front();
    }
}

fn cycle_mode(state: &mut AppState) {
    state.mode_label = match state.mode_label.as_str() {
        "SYNTHETIC STREAM" => "CALM STREAM".to_string(),
        "CALM STREAM" => "CHAOS STREAM".to_string(),
        "CHAOS STREAM" => "PULSE STORM".to_string(),
        _ => "SYNTHETIC STREAM".to_string(),
    };

    push_event(state, &format!("MODE changed to {}", state.mode_label));
}

fn local_time_short() -> String {
    Local::now().format("%I:%M:%S %p").to_string()
}

fn local_time_full() -> String {
    Local::now().format("%Y-%m-%d %I:%M:%S %p").to_string()
}

fn matrix_color(anomaly: u16) -> Color {
    match anomaly {
        0..=20 => Color::Green,
        21..=50 => Color::Yellow,
        51..=80 => Color::Magenta,
        _ => Color::Red,
    }
}

fn metric(title: &'static str, value: u16, color: Color) -> Gauge<'static> {
    Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .gauge_style(Style::default().fg(color).bg(Color::Black))
        .percent(value.min(100))
}

fn generate_matrix(
    tick: usize,
    width: usize,
    height: usize,
    energy: u16,
    anomaly: u16,
    sync_rate: u16,
) -> String {
    let safe_width = width.max(8);
    let safe_height = height.max(4);

    let density = ((energy as usize / 6) + 2).min(20);
    let chaos = ((anomaly as usize / 8) + 1).min(15);
    let coherence = ((sync_rate as usize / 10) + 3).min(14);

    let chars_calm = ["0", "1", ".", "+", "░", "▒"];
    let chars_hot = ["#", "@", "*", "▓", "█", "x", "%", "&"];

    let mut lines = Vec::new();

    for y in 0..safe_height {
        let mut line = String::new();

        for x in 0..safe_width {
            let wave_period = (24usize.saturating_sub(density)).max(4);
            let scan_period = (18usize.saturating_sub(chaos)).max(3);

            let wave = (x + tick / 2) % wave_period == 0;
            let scanline = (y + tick / 3) % scan_period == 0;
            let growth = ((x * y + tick + coherence) % 17) < density;
            let fracture = ((x + y * 2 + tick) % (22usize.saturating_sub(chaos)).max(4)) == 0;

            if fracture && anomaly > 40 {
                let idx = (x * 7 + y * 13 + tick) % chars_hot.len();
                line.push_str(chars_hot[idx]);
            } else if wave {
                line.push('█');
            } else if scanline {
                line.push('▓');
            } else if growth {
                let idx = (x * 3 + y * 5 + tick) % chars_hot.len();
                line.push_str(chars_hot[idx]);
            } else {
                let idx = (x * 5 + y * 11 + tick) % chars_calm.len();
                line.push_str(chars_calm[idx]);
            }
        }

        lines.push(line);
    }

    lines.join("\n")
}
