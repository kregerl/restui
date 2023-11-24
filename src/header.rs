use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{App, SelectionState, app::is_within_block};

pub fn render_header(f: &mut Frame, app: &mut App, chunk: &Rect) {
    let blocks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(9), // space for 7 chars and 2 for border
            Constraint::Min(0),
        ])
        .split(*chunk);

    let request_type_block = blocks[0];
    let url_block = blocks[1];

    if let Some(position) = app.last_mouse_down_event {
        if is_within_block(&request_type_block, position) {
            app.selected_input = SelectionState::RequestType;
            app.last_mouse_down_event = None;
        } else if is_within_block(&url_block, position) {
            app.selected_input = SelectionState::Url;
            app.last_mouse_down_event = None;
        }
    }

    let mut request_type_paragraph = Paragraph::new(app.selected_request_type.as_str())
        .block(Block::default().borders(Borders::ALL));

    let mut text_input = app
        .widgets
        .url_text_input
        .clone()
        .block(Block::default().borders(Borders::ALL));

    let selected_block_style = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().red());

    match app.selected_input {
        SelectionState::None => {}
        SelectionState::RequestType => {
            request_type_paragraph = request_type_paragraph.block(selected_block_style)
        }
        SelectionState::Url => {
            text_input = text_input.block(selected_block_style);
            f.set_cursor(
                url_block.x + text_input.cursor_position() as u16 + 1,
                url_block.y + 1,
            );
        }
        _ => {}
    }
    f.render_widget(request_type_paragraph, request_type_block);
    f.render_widget(text_input, url_block);
}
