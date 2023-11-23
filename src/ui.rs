use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::{ request::Request };
use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples

    let top_level_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.size());

    let req_resp_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(top_level_layout[1]);

    let mut list_state = ListState::default().with_selected(Some(app.current_request_index));

    // The List needs to know about its index
    frame.render_stateful_widget(
        http_request_list_widget(app),
        top_level_layout[0],
        &mut list_state,
    );

    frame.render_widget(http_request_widget(app), req_resp_layout[0]);
    frame.render_widget(http_response_widget(app), req_resp_layout[1]);
}

fn http_request_list_widget(app: &App) -> List {
    let items: Vec<_> = app
        .requests
        .iter()
        .map(|request| format!("{} {}", request.method(), request.uri()))
        .map(ListItem::new)
        .collect();

    List::new(items)
        .block(
            Block::default()
                .title("Requests List")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Left),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .add_modifier(Modifier::ITALIC),
        )
        .highlight_symbol("* ")
}

fn http_request_widget(app: &App) -> Paragraph {
    let display_text = app
        .requests
        .get(app.current_request_index)
        .map(format_req)
        .unwrap_or_default();

    Paragraph::new(display_text)
        .block(
            Block::default()
                .title("Request")
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default())
        .alignment(Alignment::Left)
}

fn format_req(req: &Request) -> String {
    format!("{:?} {:} {:}\n{:}\n\n{:?}", 
            req.version(),
            req.method(),
            req.uri(),
            format_headers(req),
            req.body(),
    )
}

fn format_headers(req: &Request) -> String {
    req.headers().iter()
        .map(|(key, val)| {
            format!("{}: {}", key, val.to_str().unwrap())
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn http_response_widget(app: &App) -> Paragraph {
    let display_text = app
        .requests
        .get(app.current_request_index)
        .and_then(|req| req.resp.as_ref())
        .map_or_else(String::new, |resp| format!("{:?}", resp.response));

    Paragraph::new(display_text)
        .block(
            Block::default()
                .title("Response")
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default())
        .alignment(Alignment::Left)
}
