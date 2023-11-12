use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
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

    frame.render_widget(http_request_list_widget(app), top_level_layout[0]);
    frame.render_widget(http_request_widget(app), req_resp_layout[0]);
    frame.render_widget(http_response_widget(app), req_resp_layout[1]);
}

fn http_request_list_widget(app: &mut App) -> List {
    let items = [
        ListItem::new("HTTP GET https://example.com/foo"),
        ListItem::new("HTTP POST https://example.com/bar"),
        ListItem::new("HTTP GET https://example.com/foo/bar"),
        ListItem::new("HTTP PATCH https://example.com/baz/bang/bamph"),
        ListItem::new("HTTP DELETE https://example.com/foo/bar"),
        ListItem::new("HTTP PUT https://example.com/foo/zork/quux"),
        ListItem::new("HTTP GET https://example.com/foo/zyzzx?query=string"),
        ListItem::new("HTTP OPTIONS https://example.com/foo/zork"),
        ListItem::new("HTTP GET https://example.com/foo/hello/sailor"),
        ListItem::new("HTTP POST https://example.com/foo/bar/baz/bang/bamph"),
    ];

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
    let req_content = "
GET /zork?type=team HTTP/1.1
Host: hackerone.com

Cookie: h1_device_id=178f6f86; __Host-session=dEt6aEtQ08...;
User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8
Accept-Language: en-US,en;q=0.5
Accept-Encoding: gzip, deflate, br
Upgrade-Insecure-Requests: 1
Sec-Fetch-Dest: document
Sec-Fetch-Mode: navigate
Sec-Fetch-Site: none
Sec-Fetch-User: ?1
Dnt: 1
Sec-Gpc: 1
Te: trailers
Connection: close
";

    Paragraph::new(req_content)
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
    let resp_content = "
HTTP/2 200 OK

Content-Type: text/html; charset=utf-8
Cache-Control: no-store
Content-Disposition: inline; filename='response.html'
X-Request-Id: 30125139-8cbf-4cc3-bb1f-f98f86caec3c
Set-Cookie: __Host-session=bE5hSz...
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-Xss-Protection: 1; mode=block
X-Download-Options: noopen
X-Permitted-Cross-Domain-Policies: none
Referrer-Policy: strict-origin-when-cross-origin
Expect-Ct: enforce, max-age=86400
Content-Security-Policy: default-src 'none'; base-uri 'self'; block-all-mixed-content; child-src www.yout...
Cf-Cache-Status: DYNAMIC
Server: cloudflare
Cf-Ray: 825282af6b029872-SJC

Here haz content
";

    Paragraph::new(resp_content)
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
