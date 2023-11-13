use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
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

    frame.render_stateful_widget(
        http_request_list_widget(app),
        top_level_layout[0],
        &mut list_state,
    );

    // frame.render_widget(http_request_list_widget(app), top_level_layout[0]);

    frame.render_widget(http_request_widget(app), req_resp_layout[0]);
    frame.render_widget(http_response_widget(app), req_resp_layout[1]);
}

fn http_request_list_widget(app: &mut App) -> List {
    // let items = vec![
    //     ListItem::new("HTTP GET https://example.com/foo"),
    //     ListItem::new("HTTP POST https://example.com/bar"),
    //     ListItem::new("HTTP GET https://example.com/foo/bar"),
    //     ListItem::new("HTTP PATCH https://example.com/baz/bang/bamph"),
    //     ListItem::new("HTTP DELETE https://example.com/foo/bar"),
    //     ListItem::new("HTTP PUT https://example.com/foo/zork/quux"),
    //     ListItem::new("HTTP GET https://example.com/foo/zyzzx?query=string"),
    //     ListItem::new("HTTP OPTIONS https://example.com/foo/zork"),
    //     ListItem::new("HTTP GET https://example.com/foo/hello/sailor"),
    //     ListItem::new("HTTP POST https://example.com/foo/bar/baz/bang/bamph"),
    // ];

    let items: Vec<_> = app
        .requests
        .iter()
        .map(|request| format!("HTTP {} {}", request.verb, request.domain))
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
        .highlight_symbol(">>")
}

fn http_request_widget(app: &mut App) -> Paragraph {
    let display_text: String = match app.requests.get(app.current_request_index) {
        Some(request) => request.request_body.to_string(),
        None => "".to_string(),
    };

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

fn http_response_widget(app: &mut App) -> Paragraph {
    let display_text: String = match app.requests.get(app.current_request_index) {
        Some(request) => request.response_body.to_string(),
        None => "".to_string(),
    };

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
