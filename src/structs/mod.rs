use crate::consul::fetch_nodes;
use serde::Deserialize;
use std::io;
use std::io::Write;

const SERVICES: &[&str] = &[
    "consul",
    "elasticsearch",
    "pipeline-hazelcast",
    "pipeline-hazelcast-gnmi",
    "pipeline",
    "pipeline-alert-service",
    "pipeline-config-service",
    "pipeline-config-ui",
    "pipeline-device-portal-rest-api",
    "pipeline-snpfa-service",
    "pipeline-grafana",
    "pipeline-haproxy",
    "pipeline-kibana",
    "pipeline-validation-service",
    "rproxy",
];

pub(crate) const TAB_NAMES: &[&str] = &["dev", "stage", "prod"];

pub struct CheckboxState {
    pub env: String,
    pub selected: usize,
    pub offset: usize,
    pub services: Vec<Service>,
}

pub struct AppState {
    pub error: bool,
    pub quit: bool,
    pub checkbox: CheckboxState,
    pub tab_names: Vec<String>,
    pub tab_index: usize,
    pub visible_rows: usize,
}

pub(crate) fn fetch_services(env: &str) -> anyhow::Result<Vec<Service>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(fetch_nodes(env, SERVICES.to_vec()))
}

impl CheckboxState {
    pub fn new_from_services(env: String, services: Vec<Service>) -> Self {
        Self {
            env,
            selected: 0,
            offset: 0,
            services,
        }
    }

    pub(crate) fn toggle_selected(&mut self) {
        if let Some(value) = self.services.get_mut(self.selected) {
            value.checked = !value.checked;
        }
    }

    pub(crate) fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub(crate) fn move_down(&mut self) {
        if self.selected + 1 < self.services.len() {
            self.selected += 1;
        }
    }

    pub(crate) fn scroll_into_view(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }

        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + visible_rows {
            self.offset = self.selected + 1 - visible_rows;
        }
    }

    pub(crate) fn copy_selected_to_clipboard(&mut self) {
        use cli_clipboard;
        let ips = self
            .services
            .iter()
            .filter(|s| s.checked)
            .flat_map(|s| s.ips.iter().map(|ip| ip.ip.clone()))
            .collect::<Vec<String>>()
            .join("\n");
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

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Service {
    pub checked: bool,
    pub service_name: String,
    pub ips: Vec<ServiceIP>,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ServiceIP {
    pub checked: bool,
    pub ip: String,
    pub checks: Vec<String>,
}
