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

impl<'a> StatefulWidget for CheckboxList {
    type State = CheckboxState;

    // fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    //     let title = "Services --> DEV ";
    //     let block = Block::default()
    //         .borders(Borders::ALL)
    //         .title(title)
    //         .style(Style::default().fg(Color::Green));
    //
    //     let inner = block.inner(area);
    //
    //     //let visible_rows = inner.height as usize;
    //     let mut count = 0;
    //     state.services.iter().for_each(|s| count += 1 + s.ips.len());
    //     state.visible_rows = count;
    //     block.render(area, buf);
    //
    //     let visible_rows = inner.height as usize;
    //     let start = state.offset;
    //     let end = (start + visible_rows).min(state.services.len());
    //
    //     let mut y = inner.y;
    //
    //     for (i, service) in state.services[start..end].iter().enumerate() {
    //         let index = start + i;
    //
    //         let checkbox = Checkbox::new(service.service_name.as_str(), service.checked)
    //             .style(Style::default().fg(Color::Green))
    //             .checkbox_style(
    //                 Style::default()
    //                     .fg(Color::Green)
    //                     .add_modifier(Modifier::BOLD),
    //             )
    //             .label_style(Style::default().fg(Color::Gray))
    //             .checked_symbol("✅ ")
    //             .unchecked_symbol("⬜ ");
    //
    //         let checkbox = if index == state.selected {
    //             checkbox.style(Style::default().fg(Color::LightGreen))
    //         } else {
    //             checkbox
    //         };
    //
    //         let row = Rect {
    //             x: inner.x,
    //             y,
    //             width: inner.width,
    //             height: 1,
    //         };
    //
    //         checkbox.render(row, buf);
    //
    //         let ips = to_line(service);
    //         let ips_len = ips.len() as u16;
    //
    //         let ip_row = Rect {
    //             x: inner.x,
    //             y: y + 1,
    //             width: inner.width,
    //             height: ips_len,
    //         };
    //
    //         Paragraph::new(ips)
    //             .style(Style::default().fg(Color::Green))
    //             .render(ip_row, buf);
    //
    //         y += 1 + ips_len;
    //     }
    // }

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let title = "Services --> DEV ";
        let block = Block::default().borders(Borders::ALL).title(title).style(Style::default().fg(Color::Green));
        let inner = block.inner(area);
        block.render(area, buf);

        let mut y = inner.y;

        for (i, service) in state.services.iter().enumerate() {
            if y >= inner.y + inner.height {
                break;
            }
            // let row1 = Rect {
            //     x: inner.x,
            //     y,
            //     width: inner.width,
            //     height: 1,
            // };
            // Line::raw("XXXXXX").render(row1, buf);
            // y += 1;
            let checked = state.services.get(i).unwrap().checked;
            let mut checkbox = Checkbox::new(service.service_name.as_str(), checked)
                .style(Style::default().fg(Color::Green))
                .checkbox_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .label_style(Style::default().fg(Color::Gray))
                .checked_symbol("✅ ")
                .unchecked_symbol("⬜ ");

            if i == state.selected {
                checkbox = checkbox.style(Style::default().fg(Color::LightGreen));
            }

            let row = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };

            checkbox.render(row, buf);
            let ips = to_line(service);
            let ips_len = ips.len() as u16;
            let title = Paragraph::new(ips).style(Style::default().fg(Color::Green));

            let ip_row = Rect {
                x: inner.x,
                y : y +1,
                width: inner.width,
                height: ips_len,
            };
            title.render(ip_row, buf);

            y += 1 + ips_len;
        }
    }
}

fn to_line(service: &Service) -> Vec<Line<'static>> {
    service
        .ips
        .iter()
        .map(|ip| {
            let mut spans = vec![
                Span::raw(format!("{}{:<50}"," ".repeat(10), ip.ip)), // fixed width column
                Span::raw(" "),
            ];

            for (i, status) in ip.checks.iter().enumerate() {

                match status.as_str() {
                    "passing" => spans.push(Span::styled(format!("{:<50}", "[OK]"), Style::default().fg(Color::Green))),
                    "critical" => spans.push(Span::styled(format!("{:<50}", "[X]"), Style::default().fg(Color::Red))),
                    "warning" => spans.push(Span::styled(format!("{:<50}", "[WARN]"), Style::default().fg(Color::Yellow))),
                    other => spans.push(Span::raw(other.to_string())),
                }
            }

            Line::from(spans)
        })
        .collect()
}