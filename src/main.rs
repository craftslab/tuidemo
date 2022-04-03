use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};

const TASK: [&str; 3] = ["Task1", "Task2", "Task3"];
const LOG: [(&str, &str); 3] = [("Event1", "INFO"), ("Event2", "WARN"), ("Event3", "ERROR")];

struct App<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Task", "Server"],
            index: 0,
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    #[allow(dead_code)]
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    #[allow(dead_code)]
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run(&mut terminal, app);
    if let Err(err) = res {
        println!("{:?}", err)
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Left => app.previous(),
                KeyCode::Right => app.next(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(block, size);

    let titles = app
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("tuidemo"))
        .select(app.index)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, chunks[0]);

    match app.index {
        0 => task(f, app, chunks[1]),
        1 => server(f, app, chunks[1]),
        _ => unreachable!(),
    };
}

fn task<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(4),
                Constraint::Length(30),
                Constraint::Length(30),
            ]
            .as_ref(),
        )
        .split(area);
    draw_gauge(f, app, chunks[0]);
    draw_list(f, app, chunks[1]);
    draw_text(f, app, chunks[2]);
}

fn draw_gauge<B: Backend>(f: &mut Frame<B>, _: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Progress");
    f.render_widget(block, area);

    let chunks = Layout::default()
        .constraints([Constraint::Length(2), Constraint::Length(1)].as_ref())
        .margin(1)
        .split(area);

    let progress: f64 = 0.5;
    let label = format!("{:.2}%", progress * 100.0);
    let gauge = Gauge::default()
        .block(Block::default().title("Progress:"))
        .gauge_style(
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        )
        .label(label)
        .ratio(progress);
    f.render_widget(gauge, chunks[0]);
}

fn draw_list<B: Backend>(f: &mut Frame<B>, _: &App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .direction(Direction::Horizontal)
        .split(area);
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .direction(Direction::Horizontal)
            .split(chunks[0]);

        // Draw tasks
        let mut tasks = StatefulList::with_items(TASK.to_vec());
        let t: Vec<ListItem> = tasks
            .items
            .iter()
            .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
            .collect();
        let t = List::new(t)
            .block(Block::default().borders(Borders::ALL).title("Task"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_stateful_widget(t, chunks[0], &mut tasks.state);

        // Draw logs
        let mut logs = StatefulList::with_items(LOG.to_vec());
        let info_style = Style::default().fg(Color::Blue);
        let warning_style = Style::default().fg(Color::Yellow);
        let error_style = Style::default().fg(Color::Magenta);
        let critical_style = Style::default().fg(Color::Red);
        let l: Vec<ListItem> = logs
            .items
            .iter()
            .map(|&(evt, level)| {
                let s = match level {
                    "ERROR" => error_style,
                    "CRITICAL" => critical_style,
                    "WARNING" => warning_style,
                    _ => info_style,
                };
                let content = vec![Spans::from(vec![
                    Span::styled(format!("{:<9}", level), s),
                    Span::raw(evt),
                ])];
                ListItem::new(content)
            })
            .collect();
        let l = List::new(l).block(Block::default().borders(Borders::ALL).title("Log"));
        f.render_stateful_widget(l, chunks[1], &mut logs.state);
    }
}

fn draw_text<B: Backend>(f: &mut Frame<B>, _: &App, area: Rect) {
    let text = vec![
        Spans::from("This is a paragraph with several lines. You can change style your text the way you want"),
        Spans::from(""),
        Spans::from(vec![
            Span::from("For example: "),
            Span::styled("under", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled("the", Style::default().fg(Color::Green)),
            Span::raw(" "),
            Span::styled("rainbow", Style::default().fg(Color::Blue)),
            Span::raw("."),
        ]),
        Spans::from(vec![
            Span::raw("Oh and if you didn't "),
            Span::styled("notice", Style::default().add_modifier(Modifier::ITALIC)),
            Span::raw(" you can "),
            Span::styled("automatically", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled("wrap", Style::default().add_modifier(Modifier::REVERSED)),
            Span::raw(" your "),
            Span::styled("text", Style::default().add_modifier(Modifier::UNDERLINED)),
            Span::raw(".")
        ]),
        Spans::from(
            "One more thing is that it should display unicode characters: 10€"
        ),
    ];

    let block = Block::default().borders(Borders::ALL).title("Details");
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn server<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    // TODO
}
