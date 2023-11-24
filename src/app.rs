use std::cmp::{max, min};

use ratatui::{layout::Rect, widgets::ListState};
use tui_textarea::TextArea;

use crate::text_input::TextInput;

pub struct StatefulList<T> {
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

    // fn unselect(&mut self) {
    //     self.state.select(None);
    // }

    pub fn get(&self) -> Option<&T> {
        self.items.get(self.state.selected()?)
    }
}

pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
}

impl From<&str> for RequestType {
    fn from(value: &str) -> Self {
        match value {
            "GET" => RequestType::Get,
            "POST" => RequestType::Post,
            "PUT" => RequestType::Put,
            "DELETE" => RequestType::Delete,
            "HEAD" => RequestType::Head,
            "OPTIONS" => RequestType::Options,
            _ => unreachable!("Unknown request type."),
        }
    }
}

impl RequestType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestType::Get => "GET",
            RequestType::Post => "POST",
            RequestType::Put => "PUT",
            RequestType::Delete => "DELETE",
            RequestType::Head => "HEAD",
            RequestType::Options => "OPTIONS",
        }
    }
}

#[derive(Copy, Clone)]
pub enum SelectionState {
    None,
    RequestType,
    Url,
    RequestTabs,
    RequestBody,
    Popup,
}

impl SelectionState {
    pub fn next(self) -> Self {
        match &self {
            SelectionState::None => SelectionState::RequestType,
            SelectionState::RequestType => SelectionState::Url,
            SelectionState::Url => SelectionState::RequestTabs,
            SelectionState::RequestTabs => SelectionState::RequestBody,
            SelectionState::RequestBody => SelectionState::RequestType,
            _ => self,
        }
    }

    pub fn previous(self) -> Self {
        match &self {
            SelectionState::RequestBody => SelectionState::RequestTabs,
            SelectionState::RequestTabs => SelectionState::Url,
            SelectionState::Url => SelectionState::RequestType,
            SelectionState::RequestType => SelectionState::RequestBody,
            _ => self,
        }
    }
}

pub struct TabContainer<T> {
    pub items: Vec<T>,
    pub index: usize,
}

impl<T> TabContainer<T> {
    pub fn move_left(&mut self) {
        let new_index = self.index.saturating_sub(1);
        self.index = max(new_index, 0);
    }

    pub fn move_right(&mut self) {
        let new_index = self.index.saturating_add(1);
        self.index = min(new_index, self.items.len() - 1);
    }
}

pub struct AppWidgets<'a> {
    pub url_text_input: TextInput<'a>,
    pub query_params_text_area: TextArea<'a>,
    pub headers_text_area: TextArea<'a>,
    pub body_text_area: TextArea<'a>,
    pub auth_text_area: TextArea<'a>,
}

pub struct App<'a> {
    pub widgets: AppWidgets<'a>,
    pub selected_request_type: RequestType,
    pub selected_input: SelectionState,
    pub request_types: StatefulList<&'a str>,
    pub tabs: TabContainer<&'a str>,
    pub show_request_type_popup: bool,
    pub response_text: String,
    pub last_mouse_down_event: Option<(u16, u16)>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let selected_request_type = RequestType::Get;
        Self {
            widgets: {
                AppWidgets {
                    url_text_input: TextInput::new("Url"),
                    query_params_text_area: TextArea::default(),
                    headers_text_area: TextArea::default(),
                    body_text_area: TextArea::default(),
                    auth_text_area: TextArea::default(),
                }
            },
            selected_request_type,
            selected_input: SelectionState::None,
            request_types: StatefulList::with_items(vec![
                "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS",
            ]),
            tabs: TabContainer {
                items: vec!["Query", "Headers", "Body", "Auth"],
                index: 0,
            },
            show_request_type_popup: false,
            response_text: String::new(),
            last_mouse_down_event: None,
        }
    }
}

pub fn is_within_block(block: &Rect, position: (u16, u16)) -> bool {
    let block_right = block.x + block.width;
    let block_bottom = block.y + block.height;

    position.0 >= block.x
        && position.0 <= block_right
        && position.1 >= block.y
        && position.1 <= block_bottom
}
