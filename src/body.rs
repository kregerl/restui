use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Tabs, Paragraph},
    Frame,
};
use crate::{app::{SelectionState, is_within_block}, App};

pub fn render_body(f: &mut Frame, app: &mut App, chunk: &Rect) {
    let blocks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // space for 7 chars and 2 for border
            Constraint::Percentage(50),
        ])
        .split(*chunk);

    let request_side = blocks[0];
    let response_side = blocks[1];

    render_request_side(f, app, request_side);
    render_response_side(f, app, response_side);
}

fn render_request_side(f: &mut Frame, app: &mut App, chunk: Rect) {
    let blocks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // space for 7 chars and 2 for border
            Constraint::Min(0),
        ])
        .split(chunk);

    let tab_block = blocks[0];
    let request_block = blocks[1];

    if let Some(position) = app.last_mouse_down_event {
        if is_within_block(&tab_block, position) {
            app.selected_input = SelectionState::RequestTabs;
            app.last_mouse_down_event = None;
        } else if is_within_block(&request_block, position) {
            app.selected_input = SelectionState::RequestBody;
            app.last_mouse_down_event = None;
        }
    }

    let mut tabs = Tabs::new(app.tabs.items.clone())
        .block(Block::default().borders(Borders::ALL))
        .select(app.tabs.index)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    let selected_block_style = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().red());

    // "Query", "Headers", "Body", "Auth"
    let text_area = match app.tabs.index {
        0 => &mut app.widgets.query_params_text_area,
        1 => &mut app.widgets.headers_text_area,
        2 => &mut app.widgets.body_text_area,
        3 => &mut app.widgets.auth_text_area,
        _ => unreachable!("Index out of bounds for tabs."),
    };
    text_area.set_block(Block::default().borders(Borders::ALL));

    match app.selected_input {
        SelectionState::RequestTabs => tabs = tabs.block(selected_block_style),
        SelectionState::RequestBody => text_area.set_block(selected_block_style),
        _ => {}
    }

    f.render_widget(tabs, tab_block);
    f.render_widget(text_area.widget(), request_block);
}

fn render_response_side(f: &mut Frame, app: &mut App, chunk: Rect) {
    let paragraph = Paragraph::new(app.response_text.clone()).block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, chunk);
}