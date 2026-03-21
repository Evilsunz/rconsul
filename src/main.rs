mod consul;
mod structs;
mod ui;

use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, StatefulWidget, Widget},
};
use ratatui::style::Styled;
use ratatui::widgets::{BorderType, Padding, Paragraph};
use tui_checkbox::Checkbox;
use crate::consul::fetch_nodes;
use crate::structs::{AppState, CheckboxState, Service, ServiceIP};
use crate::ui::CheckboxList;

fn main() -> anyhow::Result<()> {
    color_eyre::install().map_err(|err| anyhow::anyhow!(err))?;

    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let services = runtime.block_on(fetch_nodes("dev", vec!(
        "consul",
        // "discovery-hazelcast-5",
        // "edith-ssheventprocessor",
        "optical-mapping-mongo-config",
        "optical-mapping-mongo-query-router",
        "optical-mapping-mongo-shard",
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
    )));

    ratatui::run(|terminal| {
        let mut services = match services {
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

        // let es = {
        //     let index = services.iter().position(|service| service.service_name == "elasticsearch").unwrap_or(0);
        //     &mut services[index]
        // };
        //
        // for i in 0..30 {
        //     es.ips.push(ServiceIP{
        //         checked: false,
        //         ip: format!("127.0.0.{}", i),
        //         checks: vec!["critical".to_string()]
        //     });
        // }
        
        let mut consul_services = AppState::new(services);

        loop {
            terminal.draw(|frame| render(frame, &mut consul_services))?;

            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    other => consul_services.handle_key(other,20),
                },
                _ => {}
            }
        }
    })
}

fn render_error(frame: &mut Frame, visible: bool) {
    let constraints = [Constraint::Length(2), Constraint::Fill(1)];
    let layout = Layout::vertical(constraints);
    let [top, body] = frame.area().layout(&layout);

    let title = Paragraph::new(vec![
        Line::from("+++++ RCONSUL +++++").bold(),
        Line::from(" (Up/Down to move, Space/Enter to toggle, q to quit)"),
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

fn render(frame: &mut Frame, app: &mut AppState) {
    let constraints = [Constraint::Length(2), Constraint::Fill(1)];
    let layout = Layout::vertical(constraints);
    let [top, body] = frame.area().layout(&layout);

    let title = Paragraph::new(vec![
        Line::from("+++++ RCONSUL +++++").bold(),
        Line::from(" (Up/Down to move, Space/Enter to toggle, q to quit)")
    ]).style(Style::default().fg(Color::Green));
    frame.render_widget(title.centered(), top);

    let widget = CheckboxList{};
    frame.render_stateful_widget(widget, body, &mut app.checkbox);
}