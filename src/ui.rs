use crate::request::Request;
use crate::{
    input_state::InputState,
    layout::{LayoutState, Tab},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Renders the user interface widgets.
pub fn render(state: &mut LayoutState, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    fn req_resp_layout(rect: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rect)
    }

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.size());

    frame.render_widget(tab_list_widget(state), main_layout[0]);

    draw_input_widget(&mut state.input, frame, main_layout[2]);

    if let Some(tab) = state.current_tab() {
        let req_tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(main_layout[1]);

        frame.render_widget(tab_notes_widget(tab), req_tab_layout[0]);

        let req_resp_layout = req_resp_layout(req_tab_layout[1]);

        frame.render_widget(http_request_widget(Some(&tab.req)), req_resp_layout[0]);
        frame.render_widget(http_response_widget(Some(&tab.req)), req_resp_layout[1]);
    } else {
        let main_tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_layout[1]);

        let req_resp_layout = req_resp_layout(main_tab_layout[1]);

        let list = http_request_list_widget(state);
        frame.render_stateful_widget(list, main_tab_layout[0], state.req_idx_mut());

        frame.render_widget(http_request_widget(state.current_req()), req_resp_layout[0]);
        frame.render_widget(
            http_response_widget(state.current_req()),
            req_resp_layout[1],
        );
    }
}

fn draw_input_widget(input: &mut InputState, frame: &mut Frame, rect: Rect) {
    input.match_frame(frame);

    // so. in my old implementation of this, i did this thing with graphemes 'cause i was confused
    // and thought that all chars were just u8s. However, they are not. But I don't want to mess
    // with what works. so we're keeping the weird grapheme stuff.
    let graphemes = input.input.graphemes(true).collect::<Vec<&str>>();

    // figure out the char length of all the graphemes up until the start, and up until the end,
    // of the section that we want to render, then use them to slice the input string
    let render_start = graphemes[..input.bounds.0]
        .iter()
        .map(|s| s.len())
        .sum::<usize>();

    let render_end = graphemes[..graphemes.len().min(input.bounds.1)]
        .iter()
        .map(|s| s.len())
        .sum::<usize>();

    let render_string = &input.input[render_start..render_end];

    let input_widget = Paragraph::new(render_string).block(Block::default());
    frame.render_widget(input_widget, rect);

    // now we have to calculate exactly where the cursor should be, if we want to draw it in
    // this view.
    if input.selected {
        let cursor_x = graphemes.len() - input.right_offset;

        let before_cursor = graphemes[input.bounds.0..cursor_x]
            .iter()
            .map(|s| s.width() as u16)
            .sum::<u16>();

        frame.set_cursor(rect.x + before_cursor, rect.y + 1);
    }
}

fn http_request_list_widget<'l>(state: &LayoutState) -> List<'l> {
    let items: Vec<_> = state
        .reqs()
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

fn http_request_widget(req: Option<&Request>) -> Paragraph {
    let display_text = req.map(format_req).unwrap_or_default();

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
    format!(
        "{:?} {:} {:}\n{:}\n\n{:?}",
        req.version(),
        req.method(),
        req.uri(),
        format_headers(req),
        req.body(),
    )
}

fn format_headers(req: &Request) -> String {
    req.headers()
        .iter()
        .map(|(key, val)| format!("{}: {}", key, val.to_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n")
}

fn http_response_widget(req: Option<&Request>) -> Paragraph {
    let display_text = req
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
        .alignment(Alignment::Left)
}

fn tab_list_widget(state: &LayoutState) -> Paragraph {
    let current_tab = state.current_tab();
    let tabs = state.tabs();
    let list_text = format!(
        "{}Main |{}",
        if current_tab.is_none() { " * " } else { "   " },
        tabs.iter()
            .enumerate()
            .map(|(idx, tab)| if let Some(ref name) = tab.name {
                format!(" {name} |")
            } else {
                format!(" Request {idx} |")
            })
            .collect::<String>()
    );

    Paragraph::new(list_text)
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
        .alignment(Alignment::Left)
}

fn tab_notes_widget(tab: &Tab) -> Paragraph {
    Paragraph::new(tab.notes.as_str())
        .block(
            Block::default()
                .title("Notes")
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Left)
}
