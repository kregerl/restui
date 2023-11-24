mod app;
mod body;
mod header;
mod text_input;

use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use app::{App, RequestType, SelectionState};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseEventKind, MouseButton,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame, Terminal,
};
use reqwest::header::HeaderMap;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn close_popup(app: &mut App) {
    app.show_request_type_popup = false;
    app.selected_input = SelectionState::RequestType;
}

fn send_request<'a>(app: &App<'a>) -> reqwest::Result<String> {
    let method = match app.selected_request_type {
        RequestType::Get => reqwest::Method::GET,
        RequestType::Post => reqwest::Method::POST,
        RequestType::Put => reqwest::Method::PUT,
        RequestType::Delete => reqwest::Method::DELETE,
        RequestType::Head => reqwest::Method::HEAD,
        RequestType::Options => reqwest::Method::OPTIONS,
    };
    let client = reqwest::blocking::Client::new();

    let query_params: Vec<(&str, &str)> = app
        .widgets
        .query_params_text_area
        .lines()
        .iter()
        .filter_map(|query_param_line| query_param_line.split_once(":"))
        .collect();

    let headers: HeaderMap =
        HeaderMap::from_iter(app.widgets.headers_text_area.lines().iter().filter_map(
            |header_line| {
                let (header_key, header_value) = header_line.split_once(":")?;
                Some((header_key.parse().ok()?, header_value.parse().ok()?))
            },
        ));

    let body = app.widgets.body_text_area.lines().join("");

    let response = client
        .request(method, app.widgets.url_text_input.text())
        .query(&query_params)
        .headers(headers)
        .body(body)
        .send()?;

    let response_body = response.text()?;

    Ok(response_body)
}

fn dispatch_events_pre(key: &KeyEvent, app: &mut App) {
    if let KeyCode::Char(c) = key.code {
        if key.modifiers == KeyModifiers::CONTROL {
            match c {
                'r' => {
                    let response = send_request(&app).expect("Could not send request");
                    app.response_text = response;
                    return;
                }
                'n' => {
                    app.selected_input = app.selected_input.next();
                    return;
                }
                'p' => {
                    app.selected_input = app.selected_input.previous();
                    return;
                },
                _ => {}
            }
        }
    }

    match app.selected_input {
        SelectionState::Url => app.widgets.url_text_input.on_input(key.code),
        SelectionState::RequestBody => {
            let text_area = match app.tabs.index {
                0 => &mut app.widgets.query_params_text_area,
                1 => &mut app.widgets.headers_text_area,
                2 => &mut app.widgets.body_text_area,
                3 => &mut app.widgets.auth_text_area,
                _ => unreachable!("Index out of bounds for tabs."),
            };
            let _ = text_area.input(*key);
        }
        _ => {}
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            let event = event::read()?;

            if let Event::Mouse(mouse) = event {
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    let x = mouse.column;
                    let y = mouse.row;
                    app.last_mouse_down_event = Some((x, y));
                }
            }

            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Esc {
                        if app.show_request_type_popup {
                            close_popup(&mut app);
                        } else {
                            return Ok(());
                        }
                    }

                    dispatch_events_pre(&key, &mut app);

                    match key.code {
                        KeyCode::Enter => match app.selected_input {
                            SelectionState::RequestType => {
                                app.show_request_type_popup = true;
                                app.selected_input = SelectionState::Popup;
                            }
                            SelectionState::Popup => {
                                if let Some(request_type) = app.request_types.get() {
                                    app.selected_request_type = RequestType::from(*request_type);
                                    close_popup(&mut app);
                                }
                            }
                            SelectionState::RequestTabs => {
                                app.selected_input = app.selected_input.next();
                            }
                            _ => {}
                        },
                        KeyCode::Left => match app.selected_input {
                            SelectionState::RequestTabs => app.tabs.move_left(),
                            _ => {}
                        },
                        KeyCode::Right => match app.selected_input {
                            SelectionState::RequestTabs => app.tabs.move_right(),
                            _ => {}
                        },
                        KeyCode::Down => app.request_types.next(),
                        KeyCode::Up => app.request_types.previous(),
                        _ => {}
                    }
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn ui(f: &mut Frame, app: &mut App) {
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.size());

    header::render_header(f, app, &header_chunks[0]);
    body::render_body(f, app, &header_chunks[1]);

    if app.show_request_type_popup {
        let items: Vec<ListItem> = app
            .request_types
            .items
            .iter()
            .map(|item| {
                ListItem::new(*item).style(Style::default().fg(Color::Black).bg(Color::White))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::new().red())
                    .title("Select Request Type"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let area = centered_rect(50, 50, f.size());
        f.render_widget(Clear, area);
        f.render_stateful_widget(list, area, &mut app.request_types.state);
    }
}
