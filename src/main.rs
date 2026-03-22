mod consul;
mod structs;
mod ui;

use std::env;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{Frame, buffer::Buffer, layout::{Constraint, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, StatefulWidget, Widget}, symbols};
use ratatui::layout::Offset;
use ratatui::widgets::{BorderType, Padding, Paragraph, Tabs};
use crate::consul::fetch_nodes;
use crate::structs::{AppState, Service};
use crate::ui::CheckboxList;

const SERVICES: &[&str] = &[
    "consul",
    // "optical-mapping-mongo-config",
    // "optical-mapping-mongo-query-router",
    // "optical-mapping-mongo-shard",
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

fn main() -> anyhow::Result<()> {
    color_eyre::install().map_err(|err| anyhow::anyhow!(err))?;
    let env = env::args().nth(1).unwrap_or("dev".to_string());
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let services = runtime.block_on(fetch_nodes(&env, SERVICES.to_vec()));

    ratatui::run(|terminal| {
        let services = match services {
            Ok(services) => services,
            Err(err) => {
                let mut visible = true;
                let mut last_toggle = Instant::now();
                let blink_interval = Duration::from_millis(500);

                loop {
                    if last_toggle.elapsed() >= blink_interval {
                        visible = !visible;
                        last_toggle = Instant::now();
                    }

                    terminal.draw(|frame| {
                        render_error(frame, visible);
                    })?;

                    if event::poll(Duration::from_millis(50))? {
                        if let Event::Key(key) = event::read()? {
                            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter) {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        };
        
        let mut app = AppState::new(env, services);

        loop {
            terminal.draw(|frame| render(frame, &mut app))?;

            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    KeyCode::Tab => app.next_tab(),
                    other => app.handle_key(other, 14),
                },
                _ => {}
            }
        }
    })
}

fn render(frame: &mut Frame, app: &mut AppState) {
    let constraints = [Constraint::Length(2),Constraint::Length(1), Constraint::Fill(1)];
    let layout = Layout::vertical(constraints);
    let [top, tabs, body] = frame.area().layout(&layout);

    let title = Paragraph::new(vec![
        Line::from("+++++ RCONSUL +++++").bold(),
        Line::from(" (Up/Down to move, Space/Enter to toggle, x to copy selected service IPs to clipboard, q to quit)")
    ]).style(Style::default().fg(Color::Green));
    frame.render_widget(title.centered(), top);

    let widget = CheckboxList{};
    frame.render_stateful_widget(widget, body, &mut app.checkbox);
    render_tabs(frame, tabs + Offset::new(1, 0), app);
}

pub fn render_tabs(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let tabs = Tabs::new(app.tab_names.clone())
        .style(Style::default().fg(Color::Green))
        .select(app.tab_index)
        .divider(symbols::DOT)
        .padding(" ", " ");
    frame.render_widget(tabs, area);
}

fn render_error(frame: &mut Frame, visible: bool) {
    let constraints = [Constraint::Length(2), Constraint::Fill(1)];
    let layout = Layout::vertical(constraints);
    let [top, _body] = frame.area().layout(&layout);

    let title = Paragraph::new(vec![
        Line::from("+++++ RCONSUL +++++").bold(),
        Line::from(" (Up/Down to move, Space/Enter to toggle, x to copy selected service IPs to clipboard, q to quit)"),
    ])
        .style(Style::default().fg(Color::Green));
    frame.render_widget(title.centered(), top);

    let offline_text = if visible { "+++ OFFLINE +++" } else { "               " };

    let area = frame.area().centered(
        Constraint::Length(21),
        Constraint::Length(3),
    );

    let offline = Paragraph::new(offline_text)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK))
        .block(Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().red())
            .borders(Borders::ALL));

    frame.render_widget(offline.centered(), area);
}