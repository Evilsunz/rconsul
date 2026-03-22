use std::io;
use std::io::Write;
use crossterm::event::KeyCode;
use serde::Deserialize;
use tui_scrollview::ScrollViewState;
use crate::consul::fetch_nodes;
use crate::SERVICES;

pub struct CheckboxState {
    pub env: String,
    pub selected: usize,
    pub offset: usize,
    pub services: Vec<Service>,
}

pub struct AppState {
    pub checkbox: CheckboxState,
    pub scroll: ScrollViewState,
    pub tab_names : Vec<String>,
    pub tab_index : usize,
}

impl AppState {
    pub fn new(env: String, services: Vec<Service>) -> Self {
        let tab_names = vec!("dev".to_string(),"stage".to_string(),"prod".to_string());
        let tab_index = tab_names.iter().position(|x| x == env.as_str()).unwrap_or(0);
        Self {
            checkbox: CheckboxState::new_from_services(env, services),
            scroll: ScrollViewState::new(),
            tab_names,
            tab_index
        }
    }

    pub fn handle_key(&mut self, key: KeyCode,visible_rows: usize) {
        match key {
            KeyCode::Up => {
                self.checkbox.move_up();
                self.checkbox.scroll_into_view(visible_rows);
            },
            KeyCode::Down => {
                self.checkbox.move_down();
                self.checkbox.scroll_into_view(visible_rows);
            },
            KeyCode::Char('x') => self.checkbox.copy_selected_to_clipboard(),
            KeyCode::Enter | KeyCode::Char(' ') => self.checkbox.toggle_selected(),
            _ => {}
        }
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tab_names.len();
        let new_nev= self.tab_names.get(self.tab_index).unwrap_or(&"dev".to_string()).to_string();
        self.checkbox.env = new_nev.clone();
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let services = runtime.block_on(fetch_nodes(&new_nev, SERVICES.to_vec()));
        self.checkbox.services = services.unwrap();
    }
}

impl CheckboxState {

    pub fn new_from_services(env: String, services : Vec<Service>) -> Self {
        Self {
            env,
            selected: 0,
            offset: 0,
            services
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
        if visible_rows == 0 {
            return;
        }

        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + visible_rows {
            self.offset = self.selected + 1 - visible_rows;
        }
    }

    fn copy_selected_to_clipboard(&mut self) {
        use cli_clipboard;
        let ips = self.services.iter()
            .filter(|s| s.checked)
            .flat_map(|s| s.ips.iter().map(|ip| ip.ip.clone()))
            .collect::<Vec<String>>().join("\n");
        cli_clipboard::set_contents(ips).unwrap();
        print!("\x07");
        let _ = io::stdout().flush();
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