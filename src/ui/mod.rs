use std::io;
use std::io::Write;
use std::time::{Duration, Instant};
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui::buffer::Buffer;
use ratatui::{symbols, Frame};
use ratatui::layout::{Constraint, Layout, Offset, Rect};
use ratatui::prelude::{Color, Line, Modifier, StatefulWidget, Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Tabs, Widget};
use tui_checkbox::Checkbox;
use crate::structs::{AppState, CheckboxState, Service};

impl AppState {
    pub fn new(env: String) -> Self {
        let tab_names: Vec<String> = crate::structs::TAB_NAMES.iter().map(|tab| tab.to_string()).collect();
        let tab_index = tab_names
            .iter()
            .position(|x| x == env.as_str())
            .unwrap_or(0);
        let (services, error) = match crate::consul::fetch_services(&env) {
            Ok(services) => (services, false),
            Err(_e) => (vec!(), true)
        };
        Self {
            error,
            quit: false,
            checkbox: CheckboxState::new_from_services(env, services),
            tab_names,
            tab_index,
            visible_rows: 12,
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> anyhow::Result<()> {
        let mut visible = true;
        let mut last_toggle = Instant::now();
        let blink_interval = Duration::from_millis(500);
        while self.quit != true {
            if last_toggle.elapsed() >= blink_interval {
                visible = !visible;
                last_toggle = Instant::now();
            }
            terminal.draw(|frame| self.render(frame, visible))?;
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key.code);
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, visible: bool) {
        let constraints = [
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Fill(1),
        ];
        let layout = Layout::vertical(constraints);
        let [top, tabs, body] = frame.area().layout(&layout);

        let title = Paragraph::new(vec![
            Line::from("+++++ RCONSUL +++++").bold(),
            Line::from(" (Up/Down to move, Space/Enter to toggle, r refresh list, x to copy selected service IPs to clipboard, q to quit)")
        ]).style(Style::default().fg(Color::Green));
        frame.render_widget(title.centered(), top);
        if self.error {
            self.render_error(frame, visible);
        } else {
            let widget = CheckboxList {};
            frame.render_stateful_widget(widget, body, &mut self.checkbox);
        }
        self.render_tabs(frame, tabs + Offset::new(1, 0));
    }

    pub fn render_tabs(&mut self, frame: &mut Frame, area: Rect) {
        let tabs = Tabs::new(self.tab_names.clone())
            .style(Style::default().fg(Color::Green))
            .select(self.tab_index)
            .divider(symbols::DOT)
            .padding(" ", " ");
        frame.render_widget(tabs, area);
    }

    fn render_error(&mut self, frame: &mut Frame, visible: bool) {
        let offline_text = if visible {
            "+++ OFFLINE +++"
        } else {
            "               "
        };

        let area = frame
            .area()
            .centered(Constraint::Length(21), Constraint::Length(3));

        let offline = Paragraph::new(offline_text)
            .style(Style::default().fg(Color::Red))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().red())
                    .borders(Borders::ALL),
            );

        frame.render_widget(offline.centered(), area);
    }

    pub fn handle_key(&mut self,key_code: KeyCode) {
        match key_code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.quit = true;
            }
            KeyCode::Up => {
                self.checkbox.move_up();
                self.checkbox.scroll_into_view(self.visible_rows);
            }
            KeyCode::Down => {
                self.checkbox.move_down();
                self.checkbox.scroll_into_view(self.visible_rows);
            }
            KeyCode::Tab => self.next_tab(),
            KeyCode::Char('x') => self.checkbox.copy_selected_to_clipboard(),
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Enter | KeyCode::Char(' ') => self.checkbox.toggle_selected(),
            _ => {}
        }
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tab_names.len();
        let new_env = self
            .tab_names
            .get(self.tab_index)
            .unwrap_or(&"dev".to_string())
            .to_string();
        self.checkbox.env = new_env.clone();
        self.update_services(&new_env);
    }

    pub fn refresh(&mut self) {
        let env = self.checkbox.env.clone();
        self.update_services(env.as_str());
        print!("\x07");
        let _ = io::stdout().flush();
    }

    fn update_services(&mut self, env: &str) {
        match crate::consul::fetch_services(env) {
            Ok(services) => {
                self.error = false;
                self.checkbox.services = services;
            }
            Err(_e) => {
                self.error = true;
                self.checkbox.services = vec![];
            }
        }
    }
    
}

pub struct CheckboxList {}

impl StatefulWidget for CheckboxList {
    type State = CheckboxState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Services --> {} ", state.env))
            .style(Style::default().fg(Color::Green));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut y = inner.y;
        let bottom = inner.y + inner.height;

        for (index, service) in state.services.iter().enumerate().skip(state.offset) {
            let row_height = 1 + service.ips.len() as u16;

            if y + row_height > bottom {
                break;
            }

            let mut checkbox = Checkbox::new(service.service_name.as_str(), service.checked)
                .style(Style::default().fg(Color::Green))
                .checkbox_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .label_style(Style::default().fg(Color::Gray))
                .checked_symbol("✅ ")
                .unchecked_symbol("⬜ ");

            if index == state.selected {
                checkbox = checkbox.style(Style::default().fg(Color::LightGreen));
            }

            checkbox.render(
                Rect {
                    x: inner.x,
                    y,
                    width: inner.width,
                    height: 1,
                },
                buf,
            );

            Paragraph::new(to_line(service))
                .style(Style::default().fg(Color::Green))
                .render(
                    Rect {
                        x: inner.x,
                        y: y + 1,
                        width: inner.width,
                        height: service.ips.len() as u16,
                    },
                    buf,
                );

            y += row_height;
        }
    }
}

fn to_line(service: &Service) -> Vec<Line<'static>> {
    service
        .ips
        .iter()
        .map(|ip| {
            let mut spans = vec![
                Span::raw(format!("{}{:<50}", " ".repeat(10), ip.ip)),
                Span::raw(" "),
            ];

            for status in &ip.checks {
                match status.as_str() {
                    "passing" => spans.push(Span::styled(
                        format!("{:<50}", "[OK]"),
                        Style::default().fg(Color::Green),
                    )),
                    "critical" => spans.push(Span::styled(
                        format!("{:<50}", "[X]"),
                        Style::default().fg(Color::Red),
                    )),
                    "warning" => spans.push(Span::styled(
                        format!("{:<50}", "[WARN]"),
                        Style::default().fg(Color::Yellow),
                    )),
                    other => spans.push(Span::raw(other.to_string())),
                }
            }

            Line::from(spans)
        })
        .collect()
}