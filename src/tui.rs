use std::collections::VecDeque;
use std::io::{self, Stdout};
use std::panic;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Row, Table, TableState,
    Tabs, Wrap,
};
use ratatui::{Frame, Terminal};

use crate::lab::{
    self, load_network, save_network, GuidedCheck, GuidedStep, LabManifest, LabSummary,
};
use crate::learn::{self, LearningSession};
use crate::network::BlockchainNetwork;

const TICK_RATE: Duration = Duration::from_millis(250);
const MAX_EVENTS: usize = 100;

type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Labs,
    Tasks,
    Network,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Self::Labs => Self::Tasks,
            Self::Tasks => Self::Network,
            Self::Network => Self::Labs,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Labs => Self::Network,
            Self::Tasks => Self::Labs,
            Self::Network => Self::Tasks,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum EventLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
struct Activity {
    timestamp: String,
    level: EventLevel,
    message: String,
}

pub struct TuiApp {
    state_path: String,
    labs_root: PathBuf,
    labs: Vec<LabSummary>,
    lab_state: ListState,
    task_state: ListState,
    node_state: TableState,
    focus: Focus,
    manifest: Option<LabManifest>,
    session: Option<LearningSession>,
    network: Option<BlockchainNetwork>,
    activity: VecDeque<Activity>,
    should_quit: bool,
    show_help: bool,
    last_refresh: Instant,
}

impl TuiApp {
    pub fn new(state_path: String, labs_root: PathBuf) -> Result<Self, String> {
        let labs = lab::discover_labs(&labs_root)?;
        let mut lab_state = ListState::default();
        if !labs.is_empty() {
            lab_state.select(Some(0));
        }

        let mut app = Self {
            state_path,
            labs_root,
            labs,
            lab_state,
            task_state: ListState::default(),
            node_state: TableState::default(),
            focus: Focus::Labs,
            manifest: None,
            session: None,
            network: None,
            activity: VecDeque::new(),
            should_quit: false,
            show_help: false,
            last_refresh: Instant::now(),
        };
        app.restore_active_session();
        app.refresh_network();
        app.push_activity(EventLevel::Info, "TUI ready — select a lab and press Enter");
        Ok(app)
    }

    fn restore_active_session(&mut self) {
        let Ok(session) = learn::load_active_session(&self.state_path) else {
            return;
        };
        let Ok((manifest, _)) = lab::load_lab(&self.labs_root, &session.lab_id) else {
            return;
        };

        if let Some(index) = self.labs.iter().position(|item| item.id == session.lab_id) {
            self.lab_state.select(Some(index));
        }
        self.task_state.select(Some(
            session
                .current_step
                .min(manifest.guided_steps.len().saturating_sub(1)),
        ));
        self.manifest = Some(manifest);
        self.session = Some(session);
    }

    fn push_activity(&mut self, level: EventLevel, message: impl Into<String>) {
        if self.activity.len() >= MAX_EVENTS {
            self.activity.pop_front();
        }
        self.activity.push_back(Activity {
            timestamp: clock_time(),
            level,
            message: message.into(),
        });
    }

    fn refresh_network(&mut self) {
        self.network = load_network(&self.state_path).ok();
        if let Some(network) = &self.network {
            if network.node_count() > 0 && self.node_state.selected().is_none() {
                self.node_state.select(Some(0));
            }
        }
        self.last_refresh = Instant::now();
    }

    fn selected_lab(&self) -> Option<&LabSummary> {
        self.lab_state
            .selected()
            .and_then(|index| self.labs.get(index))
    }

    fn current_step(&self) -> Option<&GuidedStep> {
        let manifest = self.manifest.as_ref()?;
        let session = self.session.as_ref()?;
        manifest.guided_steps.get(session.current_step)
    }

    fn selected_task(&self) -> Option<&GuidedStep> {
        let manifest = self.manifest.as_ref()?;
        self.task_state
            .selected()
            .and_then(|index| manifest.guided_steps.get(index))
    }

    fn start_selected_lab(&mut self) {
        let Some(id) = self.selected_lab().map(|item| item.id.clone()) else {
            self.push_activity(EventLevel::Warning, "No lab selected");
            return;
        };
        let result = (|| {
            let (manifest, _) = lab::load_lab(&self.labs_root, &id)?;
            learn::require_guided_steps(&manifest)?;
            let (session, _) = learn::start_learning(&manifest, &self.state_path)?;
            Ok::<_, String>((manifest, session))
        })();

        match result {
            Ok((manifest, session)) => {
                self.task_state.select(Some(0));
                self.manifest = Some(manifest);
                self.session = Some(session);
                self.focus = Focus::Tasks;
                self.refresh_network();
                self.push_activity(EventLevel::Success, format!("Started guided lab: {id}"));
            }
            Err(err) => self.push_activity(EventLevel::Error, err),
        }
    }

    fn check_current_task(&mut self) {
        let Some(mut session) = self.session.take() else {
            self.push_activity(EventLevel::Warning, "Start a guided lab first");
            return;
        };
        let Some(manifest) = self.manifest.clone() else {
            self.session = Some(session);
            return;
        };
        let ack = match self.current_check(&manifest, &session) {
            Some(GuidedCheck::Acknowledge { token }) if token.is_empty() => Some("understood"),
            Some(GuidedCheck::Acknowledge { token }) => Some(token.as_str()),
            _ => None,
        };

        match learn::check_current_step(&manifest, &mut session, ack) {
            Ok(report) => {
                if report.passed {
                    self.push_activity(
                        EventLevel::Success,
                        format!("Task passed: {}", report.step_id),
                    );
                    if report.lab_completed {
                        self.push_activity(EventLevel::Success, "Lab completed");
                    } else {
                        self.task_state.select(Some(session.current_step));
                    }
                } else {
                    self.push_activity(
                        EventLevel::Warning,
                        format!("{} — {}", report.observed, report.next_action),
                    );
                }
            }
            Err(err) => self.push_activity(EventLevel::Error, err),
        }
        self.session = Some(session);
        self.refresh_network();
    }

    fn current_check<'a>(
        &self,
        manifest: &'a LabManifest,
        session: &LearningSession,
    ) -> Option<&'a GuidedCheck> {
        manifest
            .guided_steps
            .get(session.current_step)
            .map(|step| &step.check)
    }

    fn reveal_hint(&mut self) {
        let Some(mut session) = self.session.take() else {
            self.push_activity(EventLevel::Warning, "Start a guided lab first");
            return;
        };
        let Some(manifest) = self.manifest.clone() else {
            self.session = Some(session);
            return;
        };

        match learn::reveal_hint(&manifest, &mut session) {
            Ok(report) => self.push_activity(
                EventLevel::Info,
                format!("Hint L{}: {}", report.hint_level, report.hint),
            ),
            Err(err) => self.push_activity(EventLevel::Warning, err),
        }
        self.session = Some(session);
    }

    fn apply_selected_task(&mut self) {
        let Some(session) = &self.session else {
            self.start_selected_lab();
            return;
        };
        let selected = self.task_state.selected().unwrap_or(session.current_step);
        if selected != session.current_step {
            self.push_activity(
                EventLevel::Warning,
                "Tasks are sequential; select the highlighted current task",
            );
            return;
        }

        let Some(step) = self.current_step().cloned() else {
            self.push_activity(EventLevel::Info, "All tasks are complete");
            return;
        };
        let command = step.command_template.trim();
        if command.contains("tx create") {
            self.apply_transaction_command(command);
        } else if command.contains("mine once") {
            self.mine_once();
        } else {
            self.check_current_task();
        }
    }

    fn apply_transaction_command(&mut self, command: &str) {
        let parsed = parse_transaction_command(command);
        let Ok(action) = parsed else {
            self.push_activity(EventLevel::Error, parsed.unwrap_err());
            return;
        };
        let result = (|| {
            let mut network = load_network(&self.state_path)?;
            if action.sender_id >= network.node_count()
                || action.recipient_id >= network.node_count()
            {
                return Err("Invalid sender or recipient node".to_string());
            }
            let recipient = network.get_node_address(action.recipient_id);
            let amount = lab::coin_to_satoshi(action.amount_coin)?;
            let tx = network
                .create_transaction_with_fee(
                    action.sender_id,
                    &recipient,
                    amount,
                    action.fee_satoshi,
                )
                .ok_or_else(|| {
                    "Transaction rejected (balance, fee, or mempool conflict)".to_string()
                })?;
            let tx_id = tx.id.clone();
            save_network(&network, &self.state_path)?;
            Ok::<_, String>(tx_id)
        })();

        match result {
            Ok(tx_id) => self.push_activity(
                EventLevel::Success,
                format!("Transaction broadcast: {}", short_hash(&tx_id)),
            ),
            Err(err) => self.push_activity(EventLevel::Error, err),
        }
        self.refresh_network();
    }

    fn mine_once(&mut self) {
        let result = (|| {
            let mut network = load_network(&self.state_path)?;
            if network.current_val_id().is_none() {
                network.select_validator(0)?;
            }
            let before = network
                .nodes
                .first()
                .map(|node| node.blockchain.len())
                .unwrap_or(0);
            let block = network
                .mine_block()
                .ok_or_else(|| "Block could not be mined".to_string())?;
            save_network(&network, &self.state_path)?;
            Ok::<_, String>((before, block.index, block.hash))
        })();

        match result {
            Ok((before, index, hash)) => self.push_activity(
                EventLevel::Success,
                format!(
                    "Chain moved {} → {} · block {}",
                    before.saturating_sub(1),
                    index,
                    short_hash(&hash)
                ),
            ),
            Err(err) => self.push_activity(EventLevel::Error, err),
        }
        self.refresh_network();
    }

    fn move_selection(&mut self, delta: isize) {
        match self.focus {
            Focus::Labs => {
                let len = self.labs.len();
                move_list_selection(&mut self.lab_state, len, delta);
            }
            Focus::Tasks => {
                let len = self
                    .manifest
                    .as_ref()
                    .map(|manifest| manifest.guided_steps.len())
                    .unwrap_or(0);
                move_list_selection(&mut self.task_state, len, delta);
            }
            Focus::Network => {
                let len = self
                    .network
                    .as_ref()
                    .map(BlockchainNetwork::node_count)
                    .unwrap_or(0);
                move_table_selection(&mut self.node_state, len, delta);
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => self.show_help = false,
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Tab | KeyCode::Right => self.focus = self.focus.next(),
            KeyCode::BackTab | KeyCode::Left => self.focus = self.focus.previous(),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Enter => match self.focus {
                Focus::Labs => self.start_selected_lab(),
                Focus::Tasks => self.apply_selected_task(),
                Focus::Network => {}
            },
            KeyCode::Char('s') => self.start_selected_lab(),
            KeyCode::Char('a') => self.apply_selected_task(),
            KeyCode::Char('c') => self.check_current_task(),
            KeyCode::Char('h') => self.reveal_hint(),
            KeyCode::Char('m') => self.mine_once(),
            KeyCode::Char('r') => {
                self.restore_active_session();
                self.refresh_network();
                self.push_activity(EventLevel::Info, "State refreshed");
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TransactionAction {
    sender_id: usize,
    recipient_id: usize,
    amount_coin: f64,
    fee_satoshi: u64,
}

fn parse_transaction_command(command: &str) -> Result<TransactionAction, String> {
    let tokens = shell_words::split(command).map_err(|err| err.to_string())?;
    let value_after = |flag: &str| -> Option<&str> {
        tokens
            .iter()
            .position(|token| token == flag)
            .and_then(|index| tokens.get(index + 1))
            .map(String::as_str)
    };
    let sender_id = value_after("--sender-id")
        .ok_or("Missing --sender-id")?
        .parse()
        .map_err(|_| "Invalid --sender-id".to_string())?;
    let recipient_id = value_after("--recipient-id")
        .ok_or("Missing --recipient-id")?
        .parse()
        .map_err(|_| "Invalid --recipient-id".to_string())?;
    let amount_coin = value_after("--amount-coin")
        .ok_or("Missing --amount-coin")?
        .parse()
        .map_err(|_| "Invalid --amount-coin".to_string())?;
    let fee_satoshi = value_after("--fee-satoshi")
        .map(str::parse)
        .transpose()
        .map_err(|_| "Invalid --fee-satoshi".to_string())?
        .unwrap_or(1_000);
    Ok(TransactionAction {
        sender_id,
        recipient_id,
        amount_coin,
        fee_satoshi,
    })
}

fn move_list_selection(state: &mut ListState, len: usize, delta: isize) {
    if len == 0 {
        state.select(None);
        return;
    }
    let current = state.selected().unwrap_or(0) as isize;
    let next = (current + delta).clamp(0, len.saturating_sub(1) as isize) as usize;
    state.select(Some(next));
}

fn move_table_selection(state: &mut TableState, len: usize, delta: isize) {
    if len == 0 {
        state.select(None);
        return;
    }
    let current = state.selected().unwrap_or(0) as isize;
    let next = (current + delta).clamp(0, len.saturating_sub(1) as isize) as usize;
    state.select(Some(next));
}

fn clock_time() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let day_seconds = seconds % 86_400;
    format!(
        "{:02}:{:02}:{:02}",
        day_seconds / 3_600,
        (day_seconds % 3_600) / 60,
        day_seconds % 60
    )
}

fn short_hash(hash: &str) -> String {
    hash.chars().take(8).collect()
}

pub fn run(state_path: &str, labs_root: Option<&str>) -> Result<(), String> {
    let root = lab::labs_root_from(labs_root);
    let mut app = TuiApp::new(state_path.to_string(), root)?;
    let mut terminal = init_terminal().map_err(|err| err.to_string())?;

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    let result = run_loop(&mut terminal, &mut app);
    let restore_result = restore_terminal();
    result
        .and(restore_result)
        .map_err(|err| format!("TUI error: {err}"))
}

fn init_terminal() -> io::Result<TuiTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn run_loop(terminal: &mut TuiTerminal, app: &mut TuiApp) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| draw(frame, app))?;
        if event::poll(TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }
        if app.last_refresh.elapsed() >= Duration::from_secs(1) {
            app.refresh_network();
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame<'_>, app: &mut TuiApp) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(18),
            Constraint::Length(3),
        ])
        .split(frame.area());
    draw_header(frame, outer[0], app);
    draw_body(frame, outer[1], app);
    draw_footer(frame, outer[2], app);
    if app.show_help {
        draw_help(frame);
    }
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let title = Line::from(vec![
        Span::styled(
            " blockchain-sim ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("learn", Style::default().fg(Color::Cyan)),
        Span::raw(" / "),
        Span::styled(
            app.session
                .as_ref()
                .map(|session| session.lab_id.as_str())
                .unwrap_or("choose a lab"),
            Style::default().fg(Color::White),
        ),
    ]);
    let status = app
        .network
        .as_ref()
        .map(|network| {
            format!(
                "nodes {}  mempool {}  validator {}",
                network.node_count(),
                network.mempool.len(),
                network
                    .current_val_id()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "—".to_string())
            )
        })
        .unwrap_or_else(|| "no active network".to_string());
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);
    frame.render_widget(
        Paragraph::new(title).block(Block::default().borders(Borders::BOTTOM)),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(status)
            .alignment(Alignment::Right)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::BOTTOM)),
        chunks[1],
    );
}

fn draw_body(frame: &mut Frame<'_>, area: Rect, app: &mut TuiApp) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(24),
            Constraint::Percentage(50),
            Constraint::Percentage(26),
        ])
        .split(area);
    draw_navigation(frame, columns[0], app);
    draw_workspace(frame, columns[1], app);
    draw_network(frame, columns[2], app);
}

fn draw_navigation(frame: &mut Frame<'_>, area: Rect, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    let lab_items: Vec<ListItem<'_>> = app
        .labs
        .iter()
        .map(|lab| {
            let guided = lab::load_lab(&app.labs_root, &lab.id)
                .map(|(manifest, _)| !manifest.guided_steps.is_empty())
                .unwrap_or(false);
            ListItem::new(Line::from(vec![
                Span::styled(
                    if guided { "● " } else { "○ " },
                    Style::default().fg(if guided {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::raw(&lab.title),
            ]))
        })
        .collect();
    let labs_block = panel_block(" Labs ", app.focus == Focus::Labs);
    let labs = List::new(lab_items)
        .block(labs_block)
        .highlight_symbol("› ")
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(35, 45, 55))
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_stateful_widget(labs, chunks[0], &mut app.lab_state);

    let current_index = app.session.as_ref().map(|session| session.current_step);
    let completed_ids: Vec<&str> = app
        .session
        .as_ref()
        .map(|session| {
            session
                .completed_steps
                .iter()
                .map(|step| step.step_id.as_str())
                .collect()
        })
        .unwrap_or_default();
    let task_items: Vec<ListItem<'_>> = app
        .manifest
        .as_ref()
        .map(|manifest| {
            manifest
                .guided_steps
                .iter()
                .enumerate()
                .map(|(index, step)| {
                    let (icon, color) = if completed_ids.contains(&step.id.as_str()) {
                        ("✓", Color::Green)
                    } else if current_index == Some(index) {
                        ("◆", Color::Cyan)
                    } else {
                        ("·", Color::DarkGray)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{icon} "), Style::default().fg(color)),
                        Span::raw(&step.title),
                    ]))
                })
                .collect()
        })
        .unwrap_or_else(|| vec![ListItem::new("Select a guided lab")]);
    let tasks = List::new(task_items)
        .block(panel_block(" Tasks ", app.focus == Focus::Tasks))
        .highlight_symbol("› ")
        .highlight_style(Style::default().bg(Color::Rgb(35, 45, 55)).fg(Color::White));
    frame.render_stateful_widget(tasks, chunks[1], &mut app.task_state);
}

fn draw_workspace(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(42),
            Constraint::Percentage(28),
            Constraint::Percentage(30),
        ])
        .split(area);

    let progress = app
        .session
        .as_ref()
        .and_then(|session| {
            app.manifest.as_ref().map(|manifest| {
                let total = manifest.guided_steps.len().max(1);
                let completed = session.completed_steps.len();
                (completed as f64 / total as f64, completed, total)
            })
        })
        .unwrap_or((0.0, 0, 1));
    frame.render_widget(
        Gauge::default()
            .block(panel_block(" Progress ", false))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(progress.0.clamp(0.0, 1.0))
            .label(format!("{}/{} tasks", progress.1, progress.2)),
        chunks[0],
    );

    let task = app.selected_task().or_else(|| app.current_step());
    let task_text = if let Some(task) = task {
        let is_current = app
            .session
            .as_ref()
            .map(|session| {
                app.manifest
                    .as_ref()
                    .and_then(|manifest| manifest.guided_steps.get(session.current_step))
                    .map(|current| current.id == task.id)
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        Text::from(vec![
            Line::from(vec![
                Span::styled(
                    if is_current { "CURRENT  " } else { "PREVIEW  " },
                    Style::default()
                        .fg(if is_current {
                            Color::Cyan
                        } else {
                            Color::Yellow
                        })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&task.concept, Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                &task.title,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(task.instruction.as_str()),
            Line::from(""),
            Line::from(Span::styled(
                format!("$ {}", task.command_template),
                Style::default().fg(Color::Green),
            )),
            Line::from(""),
            Line::from(Span::styled(
                &task.discussion_prompt,
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )),
        ])
    } else {
        Text::from(vec![
            Line::from("Choose a guided lab from the left."),
            Line::from("Press Enter to start it."),
        ])
    };
    frame.render_widget(
        Paragraph::new(task_text)
            .wrap(Wrap { trim: true })
            .block(panel_block(" Task workspace ", true)),
        chunks[1],
    );

    draw_chain(frame, chunks[2], app);
    draw_activity(frame, chunks[3], app);
}

fn draw_chain(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(network) = &app.network else {
        frame.render_widget(
            Paragraph::new("Start a lab to create a chain.")
                .block(panel_block(" Chain movement ", false)),
            area,
        );
        return;
    };
    let Some(node) = network.nodes.first() else {
        return;
    };
    let mut lines = vec![Line::from(Span::styled(
        format!(
            "height {} · difficulty {} · mempool {}",
            node.blockchain.len().saturating_sub(1),
            network.difficulty,
            network.mempool.len()
        ),
        Style::default().fg(Color::DarkGray),
    ))];
    let visible: Vec<_> = node.blockchain.iter().rev().take(6).collect();
    let mut chain_spans = Vec::new();
    for (position, block) in visible.iter().rev().enumerate() {
        if position > 0 {
            chain_spans.push(Span::styled(" ──▶ ", Style::default().fg(Color::DarkGray)));
        }
        let is_tip = position + 1 == visible.len();
        chain_spans.push(Span::styled(
            format!("[ #{} {} ]", block.index, short_hash(&block.hash)),
            Style::default()
                .fg(if is_tip { Color::Cyan } else { Color::White })
                .add_modifier(if is_tip {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(chain_spans));
    lines.push(Line::from(""));
    if let Some(tip) = node.blockchain.last() {
        lines.push(Line::from(vec![
            Span::styled("tip ", Style::default().fg(Color::DarkGray)),
            Span::raw(short_hash(&tip.hash)),
            Span::styled("  tx ", Style::default().fg(Color::DarkGray)),
            Span::raw(tip.transactions.len().to_string()),
        ]));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(panel_block(" Chain movement ", false)),
        area,
    );
}

fn draw_activity(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let max_lines = area.height.saturating_sub(2) as usize;
    let lines: Vec<Line<'_>> = app
        .activity
        .iter()
        .rev()
        .take(max_lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|entry| {
            let (icon, color) = match entry.level {
                EventLevel::Info => ("●", Color::Blue),
                EventLevel::Success => ("✓", Color::Green),
                EventLevel::Warning => ("!", Color::Yellow),
                EventLevel::Error => ("×", Color::Red),
            };
            Line::from(vec![
                Span::styled(
                    format!("{} ", entry.timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("{icon} "), Style::default().fg(color)),
                Span::raw(&entry.message),
            ])
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel_block(" Activity ", false)),
        area,
    );
}

fn draw_network(frame: &mut Frame<'_>, area: Rect, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);
    let rows: Vec<Row<'_>> = app
        .network
        .as_ref()
        .map(|network| {
            network
                .nodes
                .iter()
                .map(|node| {
                    Row::new(vec![
                        format!("{}{}", node.id, if node.is_validator { "*" } else { "" }),
                        format!("{:.2}", node.wallet.get_balance() as f64 / 100_000_000.0),
                        node.blockchain.len().saturating_sub(1).to_string(),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();
    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Percentage(60),
            Constraint::Length(7),
        ],
    )
    .header(Row::new(vec!["Node", "Balance", "Height"]).style(Style::default().fg(Color::DarkGray)))
    .block(panel_block(" Network ", app.focus == Focus::Network))
    .highlight_style(Style::default().bg(Color::Rgb(35, 45, 55)))
    .highlight_symbol("› ");
    frame.render_stateful_widget(table, chunks[0], &mut app.node_state);

    let mempool_items: Vec<ListItem<'_>> = app
        .network
        .as_ref()
        .map(|network| {
            network
                .mempool
                .iter()
                .map(|tx| {
                    ListItem::new(Line::from(vec![
                        Span::styled("◌ ", Style::default().fg(Color::Yellow)),
                        Span::raw(short_hash(&tx.id)),
                        Span::styled(
                            format!("  {} out", tx.outputs.len()),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]))
                })
                .collect()
        })
        .unwrap_or_default();
    let mempool = if mempool_items.is_empty() {
        List::new(vec![ListItem::new(Span::styled(
            "empty",
            Style::default().fg(Color::DarkGray),
        ))])
    } else {
        List::new(mempool_items)
    };
    frame.render_widget(mempool.block(panel_block(" Mempool ", false)), chunks[1]);
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let tabs = Tabs::new(vec![
        Line::from(" Tab focus "),
        Line::from(" ↑↓ navigate "),
        Line::from(" Enter apply "),
        Line::from(" c check "),
        Line::from(" h hint "),
        Line::from(" m mine "),
        Line::from(" ? help "),
        Line::from(" q quit "),
    ])
    .style(Style::default().fg(Color::DarkGray))
    .highlight_style(Style::default().fg(Color::Cyan))
    .select(match app.focus {
        Focus::Labs => 0,
        Focus::Tasks => 2,
        Focus::Network => 1,
    })
    .block(Block::default().borders(Borders::TOP));
    frame.render_widget(tabs, area);
}

fn draw_help(frame: &mut Frame<'_>) {
    let area = centered_rect(70, 75, frame.area());
    frame.render_widget(Clear, area);
    let help = Text::from(vec![
        Line::from(Span::styled(
            "Keyboard",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        key_line("Tab / ← →", "Move focus between Labs, Tasks, Network"),
        key_line("↑ / ↓ · j / k", "Move selection"),
        key_line("Enter / a", "Start selected lab or apply current task"),
        key_line("c", "Check current task"),
        key_line("h", "Reveal the next hint"),
        key_line("m", "Mine one block"),
        key_line("r", "Reload state from disk"),
        key_line("? / Esc", "Close this help"),
        key_line("q / Ctrl-C", "Quit safely"),
        Line::from(""),
        Line::from(Span::styled(
            "Tasks remain sequential: older and future tasks can be previewed, but only the current task can be applied.",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    frame.render_widget(
        Paragraph::new(help).wrap(Wrap { trim: true }).block(
            Block::default()
                .title(" Help ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        area,
    );
}

fn key_line<'a>(key: &'a str, description: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("{key:<16}"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(description),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn panel_block(title: &str, focused: bool) -> Block<'_> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if focused {
            Color::Cyan
        } else {
            Color::Rgb(55, 65, 75)
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use std::path::Path;

    #[test]
    fn transaction_template_is_parsed() {
        let action = parse_transaction_command(
            "tx create --sender-id 0 --recipient-id 1 --amount-coin 10 --fee-satoshi 5000",
        )
        .expect("parse");
        assert_eq!(
            action,
            TransactionAction {
                sender_id: 0,
                recipient_id: 1,
                amount_coin: 10.0,
                fee_satoshi: 5_000,
            }
        );
    }

    #[test]
    fn list_selection_stays_in_bounds() {
        let mut state = ListState::default();
        state.select(Some(0));
        move_list_selection(&mut state, 3, -1);
        assert_eq!(state.selected(), Some(0));
        move_list_selection(&mut state, 3, 10);
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn app_renders_on_minimum_dashboard_size() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("labs");
        let dir = std::env::temp_dir().join(format!("tui_render_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let state = dir.join("state.json").to_string_lossy().to_string();
        let mut app = TuiApp::new(state, root).expect("app");
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal.draw(|frame| draw(frame, &mut app)).expect("draw");
        let _ = std::fs::remove_dir_all(dir);
    }
}
