use std::iter;
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::{Color, Line, Modifier, StatefulWidget, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use tui_checkbox::Checkbox;
use tui_scrollview::{ScrollView, ScrollViewState};
use crate::structs::{CheckboxState, Service};

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