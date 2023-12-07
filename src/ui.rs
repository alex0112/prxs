use crate::{
    input_state::InputState,
    layout::{LayoutState, MainPane, Pane, Tab, TabPane},
    request::Request,
    response_waiter::RequestResponse,
};
use http::{HeaderMap, HeaderValue};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, Clear, List, ListItem, Paragraph,
    },
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

// it would be nice to make this a static but alas
#[inline]
fn sel_style() -> Style {
    Style::default().fg(Color::Cyan)
}

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

    let pane = state.current_pane();

    if let Some(tab) = state.current_tab() {
        let req_tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(main_layout[1]);

        frame.render_widget(tab_notes_widget(tab, pane), req_tab_layout[0]);

        let req_resp_layout = req_resp_layout(req_tab_layout[1]);

        frame.render_widget(
            http_request_widget(Some(&tab.req), pane),
            req_resp_layout[0],
        );
        frame.render_widget(
            http_response_widget(Some(&tab.req), pane),
            req_resp_layout[1],
        );
    } else {
        let main_tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_layout[1]);

        let req_resp_layout = req_resp_layout(main_tab_layout[1]);

        frame.render_widget(
            http_request_widget(state.current_req(), pane),
            req_resp_layout[0],
        );
        frame.render_widget(
            http_response_widget(state.current_req(), pane),
            req_resp_layout[1],
        );

        let list = http_request_list_widget(state);
        frame.render_stateful_widget(list, main_tab_layout[0], state.req_idx_mut());
    }

    if let Some(ref msg) = state.err_msg {
        show_popup("Error", msg.as_str(), frame);
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

    let mut block = Block::default()
        .title("Requests List")
        .borders(Borders::ALL)
        .title(
            Title::from("a")
                .position(Position::Bottom)
                .alignment(Alignment::Right),
        );

    if matches!(
        state.current_pane(),
        Pane::Main {
            pane: MainPane::ReqList,
            ..
        }
    ) {
        block = block.border_style(sel_style());
    }

    List::new(items)
        .block(block)
        // .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .add_modifier(Modifier::ITALIC),
        )
        .highlight_symbol("* ")
}

fn http_request_widget<'p>(req: Option<&Request>, selected: &Pane) -> Paragraph<'p> {
    let display_text = req.map(format_req).unwrap_or_default();

    let mut block = Block::default()
        .title("Request")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(
            Title::from("w")
                .position(Position::Bottom)
                .alignment(Alignment::Right),
        );

    let scroll = match selected {
        Pane::Main {
            pane: MainPane::Req,
            scroll,
        }
        | Pane::Tab {
            pane: TabPane::Req,
            scroll,
            ..
        } => Some(scroll),
        _ => None,
    };

    if scroll.is_some() {
        block = block.border_style(sel_style());
    }

    let mut para = Paragraph::new(display_text).block(block);

    if let Some(s) = scroll {
        para = para.scroll((*s as u16, 0));
    }

    para
}

fn format_req(req: &Request) -> String {
    format!(
        "{:?} {} {}\n{}\n\n{:?}",
        req.version(),
        req.method(),
        req.uri(),
        format_headers(req.headers()),
        req.body(),
    )
}

fn format_headers(headers: &HeaderMap<HeaderValue>) -> String {
    headers
        .iter()
        .map(|(key, val)| format!("{}: {}", key, val.to_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n")
}

fn format_resp(resp: &RequestResponse) -> String {
    match resp.response.as_ref() {
        Err(e) => format!("Couldn't get response: {e}"),
        Ok(resp) => {
            let body = &resp.body;
            // we have to call this outside of the `unwrap_or` so that we don't reference a
            // temporary :(
            let dbg_body = format!("{body:?}");
            format!(
                "{:?}\n{}\n\n{}",
                resp.status,
                format_headers(&resp.headers),
                // if it can be a string, that's nice
                std::str::from_utf8(body).unwrap_or(dbg_body.as_str())
            )
        }
    }
}

fn http_response_widget<'p>(req: Option<&Request>, selected: &Pane) -> Paragraph<'p> {
    let display_text = req
        .and_then(|req| req.resp.as_ref())
        .map_or_else(String::new, format_resp);

    let mut block = Block::default()
        .title("Response")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(
            Title::from("s")
                .position(Position::Bottom)
                .alignment(Alignment::Right),
        );

    let scroll = match selected {
        Pane::Main {
            pane: MainPane::Resp,
            scroll,
        }
        | Pane::Tab {
            pane: TabPane::Resp,
            scroll,
            ..
        } => Some(scroll),
        _ => None,
    };

    if scroll.is_some() {
        block = block.border_style(sel_style());
    }

    let mut para = Paragraph::new(display_text).block(block);

    if let Some(s) = scroll {
        para = para.scroll((*s as u16, 0));
    }

    para
}

fn tab_list_widget(state: &LayoutState) -> Paragraph {
    let current_tab = state.current_tab();
    let tabs = state.tabs();
    let list_text = format!(
        " {}Main |{}",
        if current_tab.is_none() { "* " } else { "" },
        tabs.iter()
            .enumerate()
            .map(|(idx, tab)| {
                let mut name = if let Some(ref name) = tab.name {
                    format!(" {name} |")
                } else {
                    format!(" Request {idx} |")
                };
                if state.current_tab_idx() == Some(idx) {
                    name.insert_str(0, " *");
                }
                name
            })
            .collect::<String>()
    );

    Paragraph::new(list_text)
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
        .alignment(Alignment::Left)
}

fn tab_notes_widget<'t>(tab: &'t Tab, selected: &Pane) -> Paragraph<'t> {
    let mut block = Block::default()
        .title("Notes")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    if matches!(
        selected,
        Pane::Tab {
            pane: TabPane::Notes,
            ..
        }
    ) {
        block = block.border_style(sel_style());
    }

    Paragraph::new(tab.notes.as_str()).block(block)
}

fn show_popup(title: &str, msg: &str, frame: &mut Frame) {
    let screen = frame.size();

    // Need to add 2 for the borders
    let height = msg.lines().count().min(screen.height as usize / 2) + 2;
    let width = msg
        .lines()
        .map(str::len)
        .max()
        .unwrap_or(0)
        .min(screen.width as usize / 2)
        + 2;

    let rect = centered_rect(width as u16, height as u16, screen);

    let shadow = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::QuadrantInside);

    // offset it to the bottom right
    let shadow_rect = Rect::new(rect.x + 1, rect.y + 1, rect.width, rect.height);

    frame.render_widget(Clear, shadow_rect);
    frame.render_widget(shadow, shadow_rect);

    let paragraph = Paragraph::new(msg).block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(Clear, rect);
    frame.render_widget(paragraph, rect);
}

/// helper function to create a centered rect using up certain percentage of the
/// available Rect `r`, mostly copied from ratatui's examples
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let vert_border = (r.height - height) / 2;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vert_border),
            Constraint::Length(height),
            Constraint::Length(vert_border),
        ])
        .split(r);

    let horiz_border = (r.width - width) / 2;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(horiz_border),
            Constraint::Length(width),
            Constraint::Length(horiz_border),
        ])
        .split(popup_layout[1])[1]
}
