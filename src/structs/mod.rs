use crossterm::event::KeyCode;
use serde::Deserialize;

pub struct CheckboxState {
    pub selected: usize,
    pub offset: usize,
    pub visible_rows: usize,
    pub services: Vec<Service>,
}

impl CheckboxState {

    pub fn new_from_services(services : Vec<Service>) -> Self {
        Self {
            selected: 0,
            offset: 0,
            visible_rows: 0,
            services
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                self.move_up();
                self.scroll_into_view(self.visible_rows);
            }
            KeyCode::Down => {
                self.move_down();
                self.scroll_into_view(self.visible_rows);
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.toggle_selected(),
            _ => {}
        }
    }

    fn toggle_selected(&mut self) {
        if let Some(value) = self.services.get_mut(self.selected) {
            value.checked = !value.checked;
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected + 1 < self.services.len() {
            self.selected += 1;
        }
    }

    fn scroll_into_view(&mut self, visible_rows: usize) {
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + visible_rows {
            self.offset = self.selected + 1 - visible_rows;
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConsulEntry {
    #[serde(rename = "Node")]
    pub node: ConsulNode,
    #[serde(rename = "Checks")]
    pub checks: Vec<ConsulCheck>,
}

#[derive(Debug, Deserialize)]
pub struct ConsulNode {
    #[serde(rename = "Address")]
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct ConsulCheck {
    #[serde(rename = "Status")]
    pub status: String,
}

#[derive(Debug,Ord, PartialOrd, Eq, PartialEq)]
pub struct Service {
    pub checked: bool,
    pub service_name: String,
    pub ips: Vec<ServiceIP>
}

#[derive(Debug,Ord, PartialOrd, Eq, PartialEq)]
pub struct ServiceIP {
    pub checked: bool,
    pub ip: String,
    pub checks: Vec<String>,
}