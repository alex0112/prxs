use crossterm::event::KeyCode;
use ratatui::Frame;
use std::{borrow::ToOwned, fs, path::PathBuf};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub struct InputState {
    /// the text that is input to this view
    pub input: String,
    /// the substring of the input that is shown
    pub bounds: (usize, usize),
    /// the cursor's offset from the right side of the input
    pub right_offset: usize,
    /// last width that the view recorded. Since input views are always one line,
    /// height changes don't affect them.
    pub last_width: usize,
    /// last commands that were input, so that you can tab up through them
    pub last_commands: Vec<String>,
    /// how far tabbed up through the most recent commands you are
    pub tabbed_up: Option<u16>,
    /// Whether or not this view is selected; if we're currently typing in it
    pub selected: bool,
}

impl InputState {
    pub fn match_frame(&mut self, frame: &Frame) {
        // if it's not the same width, the terminal has been resized. Reset some
        // stuff so that everything doesn't look weird when you try to draw it.
        let width = frame.size().width as usize;
        if self.last_width != width {
            self.last_width = width;

            self.bounds.1 = self.input.as_str().width() - self.right_offset;
            self.bounds.0 = self.bounds.1.saturating_sub(self.last_width);
        }
    }

    pub fn route_keycode(&mut self, code: KeyCode) -> Option<InputCommand> {
        // just decide to which function the specified keycode should go
        match code {
            KeyCode::Backspace => self.handle_backspace(),
            KeyCode::Esc => self.handle_escape(),
            KeyCode::Tab => self.handle_tab(),
            KeyCode::Up => self.change_command(true),
            KeyCode::Down => self.change_command(false),
            KeyCode::Left => self.scroll(false, 1),
            KeyCode::Right => self.scroll(true, 1),
            KeyCode::Char(c) => self.append_char(c),
            KeyCode::Enter => return self.entered_command(),
            _ => (),
        }

        None
    }

    pub fn append_char(&mut self, ch: char) {
        // input it at the specified place
        // also have to work with unicode here so that we don't
        // insert in the middle of a utf char
        let mut graph = self.input.graphemes(true).collect::<Vec<&str>>();
        let len = graph.len();

        let ch_str = ch.to_string();
        graph.insert(len - self.right_offset, &ch_str);
        self.input = graph.join("");

        // scroll 0. This ensures that the string will display nicely when redrawn
        self.scroll(true, 0);
    }

    pub fn handle_escape(&mut self) {
        self.input = String::new();
        self.right_offset = 0;

        // once again, makes sure that the input will display nicely when redrawn
        self.scroll(false, 0);

        self.selected = false;
    }

    pub fn handle_backspace(&mut self) {
        // have to handle this all as unicode so that people can backspace
        // a whole unicode character
        let mut graph = self.input.graphemes(true).collect::<Vec<&str>>();
        let len = graph.len();

        if len > self.right_offset {
            graph.remove(len - self.right_offset - 1);
            self.input = graph.join("");
        }

        // same
        self.scroll(false, 0);
    }

    pub fn handle_tab(&mut self) {
        // if the first 3 characters are `:s ` or `:S `, then they're pressing tab to get file
        // path completion for saving the session. Handle that separately.
        if self.input.starts_with(":s ") || self.input.starts_with(":S ") {
            self.handle_tab_completion(3);
        } else {
            self.input.push('\t');
        }
    }

    pub fn get_typed_file(input: &str) -> String {
        // parse the string that is input and get the first partial file they're currently typed
        // out. We have to use special parsing for this so that people can escape spaces with
        // backslashes and quotes
        let mut in_quotes = false;
        let mut escaped = false;
        let mut done = false;

        input
            .chars()
            .filter(|c| match c {
                _ if done => false,
                #[cfg(not(windows))]
                '\\' if !escaped => {
                    escaped = true;
                    false
                }
                ' ' if !(escaped || in_quotes) => {
                    done = true;
                    false
                }
                '"' if !escaped => {
                    in_quotes = !in_quotes;
                    false
                }
                _ => {
                    escaped = false;
                    true
                }
            })
            .collect::<String>()
    }

    pub fn handle_tab_completion(&mut self, start_idx: usize) {
        // stole this from text-completions:
        // https://docs.rs/text-completions/latest/src/text_completions/lib.rs.html#65-102
        // technically this doesn't account for the fact that macos file names can have path
        // separators in them, but anyone who does that is stupid so we're not gonna support them
        fn completed(input: &str) -> Option<String> {
            let mut path = PathBuf::from(input);

            if path.is_dir() {
                return if input.ends_with('/') {
                    None
                } else {
                    let mut str = input.to_string();
                    str.push('/');
                    Some(str)
                };
            }

            let parent = path.parent().unwrap_or(&path);
            if !parent.try_exists().ok()? {
                return None;
            }

            let base_name = path.file_name()?.to_str()?;
            let dir = fs::read_dir(parent).ok()?;

            for ent in dir {
                let ent = ent.ok()?;
                let file_name = ent.file_name();
                let Some(name) = file_name.to_str() else {
                    continue;
                };

                if name.starts_with(base_name) {
                    if ent.path().is_dir() {
                        path.push(name);
                    } else {
                        path.set_file_name(name);
                    }
                    break;
                }
            }

            path.to_str().map(ToOwned::to_owned)
        }

        // So this is my messy attempt at tab completion. It actually works ok-ish
        // I think it works on Windows but I can't say for certain

        // this gets a list of the currently input attachments,
        // with support for escaping spaces with backslashes and quotes
        let incomplete = Self::get_typed_file(&self.input[start_idx..]);

        if let Some(new_input) = completed(&incomplete) {
            self.input = new_input;
        }
    }

    pub fn scroll(&mut self, right: bool, distance: usize) {
        // this is the actual scrolling part

        let graphemes = self.input.graphemes(true).collect::<Vec<&str>>();
        let len = graphemes.len();
        let display_len = self.input.width();

        if right {
            self.right_offset = self.right_offset.saturating_sub(distance);
        } else {
            self.right_offset = len.min(self.right_offset + distance);
        }

        // and this is the part that handles setting other variables to make sure
        // it displays nicely on the next redraw. Just suffice it to say this
        // handles setting all these parameters to the correct values for the input
        // field to be pretty

        //1. If the displayed width is less than last_width, just shove it all in.
        //2. Else, make sure the cursor is in screen.
        //  a. If the cursor would be to the right of the displayed section currently set by
        //     bounds, make the rightmost bound at the cursor and the leftmost accomodate
        //  b. If the cursor would be to the left, shift bounds to the left until the cursor is
        //     in view AND text fills the whole area
        //  c. If the cursor would be in the middle:
        //     i. If bounds.0 > 0, shift left until text fills whole area
        //     ii. If bounds.1 < len, shift right until text fills whole area

        if display_len < self.last_width {
            self.bounds = (0, len);
        } else {
            let reset_0 = |bounds: &mut (usize, usize), max_width: usize| {
                let mut total_width = 0;
                bounds.0 = bounds.1
                    - graphemes[..bounds.1]
                        .iter()
                        .rev()
                        .take_while(|elem| {
                            total_width += elem.width();
                            total_width < max_width
                        })
                        .count();
            };

            let left_offset = len - self.right_offset;
            let bound_0_to_cursor_width = graphemes[self.bounds.0..left_offset]
                .iter()
                .map(|s| s.width())
                .sum::<usize>();

            if bound_0_to_cursor_width >= self.last_width {
                self.bounds.1 = left_offset;
                reset_0(&mut self.bounds, self.last_width);
            } else if left_offset < self.bounds.0 {
                self.bounds.1 = len.min(left_offset + self.last_width);
                reset_0(&mut self.bounds, self.last_width);
            } else if self.bounds.0 > 0 && self.bounds.1 >= len {
                self.bounds.1 = len;
                reset_0(&mut self.bounds, self.last_width);
            }
        }
    }

    pub fn change_command(&mut self, up: bool) {
        // this handles tabbing up through recent commands
        if up {
            // if tabbing up, to older commands
            match self.tabbed_up {
                None => {
                    if !self.last_commands.is_empty() {
                        // if we haven't tabbed up at all, set it to 0 and grab the command
                        self.tabbed_up = Some(0);
                        self.input = self.last_commands[0].clone();
                    }
                }
                Some(tu) => {
                    if let Some(cmd) = self.last_commands.get(tu as usize + 1) {
                        // if we tabbed up and we can still do so more, do so.
                        self.tabbed_up = Some(tu + 1);
                        self.input = cmd.clone();
                    }
                }
            }
        } else if let Some(tab) = self.tabbed_up {
            // if tabbing down, to more recent commands
            // only do something if we've already tabbed up somewhat
            if tab == 0 {
                // if it's 0, reset the input to nothing.
                self.input = String::new();
                self.tabbed_up = None;
            } else {
                // else just go one further down the list
                self.input = self.last_commands[tab as usize - 1].clone();
                self.tabbed_up = Some(tab - 1);
            }
        }

        self.scroll(false, 0);
    }
}

pub enum InputCommand {
    SaveSession(String),
    SelectTab(usize),
    Quit,
}

impl InputState {
    fn entered_command(&mut self) -> Option<InputCommand> {
        let taken_input = std::mem::take(&mut self.input);
        // just reset the input so all variables are nice
        self.scroll(false, 0);
        self.selected = false;

        let mut sections = taken_input.split(' ');
        let Some(cmd) = sections.next() else {
            return None;
        };

        match cmd {
            // I don't like allocating new strings where avoidable, but I can't figure out a way
            // to avoid this one that's not really stupid and hacky
            ":s" | ":S" => sections.next().map(|s| InputCommand::SaveSession(s.into())),
            ":q" | ":Q" => Some(InputCommand::Quit),
            i if !i.is_empty() => {
                if !i.starts_with(':') {
                    return None;
                }

                i[1..].parse::<usize>().map(InputCommand::SelectTab).ok()
            }
            _ => None,
        }
    }
}
