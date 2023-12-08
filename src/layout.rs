use crate::{
    input_state::InputState, request::Request, response_waiter::RequestResponse, ProxyInteraction,
};
use hyper::Body;
use ratatui::widgets::ListState;
use tokio::sync::oneshot;
use uuid::Uuid;

pub enum MainPane {
    ReqList,
    Req,
    Resp,
}

impl TryFrom<char> for MainPane {
    type Error = ();
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'a' => Ok(MainPane::ReqList),
            'w' => Ok(MainPane::Req),
            's' => Ok(MainPane::Resp),
            _ => Err(()),
        }
    }
}

pub enum TabPane {
    Notes,
    Req,
    Resp,
}

impl TryFrom<char> for TabPane {
    type Error = ();
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'a' => Ok(TabPane::Notes),
            'w' => Ok(TabPane::Req),
            's' => Ok(TabPane::Resp),
            _ => Err(()),
        }
    }
}

pub enum Pane {
    Main {
        scroll: usize,
        pane: MainPane,
    },
    Tab {
        idx: usize,
        scroll: usize,
        pane: TabPane,
    },
}

impl Default for Pane {
    fn default() -> Self {
        Pane::Main {
            scroll: 0,
            pane: MainPane::ReqList,
        }
    }
}

/// Records what the current state of the UI is, e.g. what tab the user is currently viewing,
/// what request is assigned to each tab, etc.
#[derive(Default)]
pub struct LayoutState {
    requests: Vec<Request>,
    current_req_idx: ListState,
    tabs: Vec<Tab>,
    pub input: InputState,
    current_pane: Pane,
    pub err_msg: Option<String>,
}

impl LayoutState {
    /// getter for `current_req_idx` so we can pass it into a stateful widget
    pub fn req_idx_mut(&mut self) -> &mut ListState {
        &mut self.current_req_idx
    }

    /// getter for list of requests to render
    pub fn reqs(&self) -> &[Request] {
        &self.requests
    }

    /// Get the current request, if any. This should return Some if requests is non-empty, but I
    /// guess we can't guarantee. But it will return None if none have come in
    pub fn current_req(&self) -> Option<&Request> {
        match self.current_pane {
            Pane::Tab { idx, .. } => self.tabs.get(idx).map(|t| &t.req),
            Pane::Main { .. } => self
                .current_req_idx
                .selected()
                .and_then(|idx| self.requests.get(idx)),
        }
    }

    /// Same, just if you want to do something to it
    pub fn current_req_mut(&mut self) -> Option<&mut Request> {
        match self.current_pane {
            Pane::Tab { idx, .. } => self.tabs.get_mut(idx).map(|t| &mut t.req),
            Pane::Main { .. } => self
                .current_req_idx
                .selected()
                .and_then(|idx| self.requests.get_mut(idx)),
        }
    }

    /// Append a request to the list of currently stored requests and select it if it's now the
    /// only request
    pub fn add_request(
        &mut self,
        req: hyper::Request<Body>,
        sender: oneshot::Sender<ProxyInteraction>,
    ) {
        self.requests.push(Request {
            id: Uuid::new_v4(),
            interaction_tx: Some(sender),
            inner: req,
            resp: None,
        });
        // If a new request comes in but we're already viewing one, just append it. Don't mess up
        // what we're already focusing on.
        if self.current_req_idx.selected().is_none() {
            self.current_req_idx.select(Some(self.requests.len() - 1));
        }
    }

    pub fn handle_req_response(&mut self, resp: RequestResponse) {
        let Some(req) = self.requests.iter_mut().find(|r| r.id == resp.id) else {
            self.show_error(format!(
                "Couldn't handle request response: no request for id {}",
                resp.id
            ));
            return;
        };

        req.store_response(resp);
    }

    /// An internal function for jumping around the list of requests by a certain amount
    /// Doesn't move the selector if there are no requests, and selects the request at the
    /// current index offset by `amt`, if that index is within bounds of the list of requests. If
    /// it's not, it just selects req 0 or requests.len() - 1, whichever is closer to the
    /// expected one.
    fn mod_req_idx_by(&mut self, amt: isize) {
        match self.current_pane {
            Pane::Main {
                pane: MainPane::ReqList,
                ..
            } => {
                if self.requests.is_empty() {
                    return;
                }

                let selected = self.current_req_idx.selected().map_or(0, |sel| {
                    (sel as isize + amt)
                        .try_into()
                        .unwrap_or(0)
                        .min(self.requests.len() - 1)
                });

                self.current_req_idx.select(Some(selected));
            }
            Pane::Main { ref mut scroll, .. } | Pane::Tab { ref mut scroll, .. } => {
                *scroll = (*scroll as isize + amt).try_into().unwrap_or(0);
            }
        }
    }

    /// Select the next request in the list, if it can. Otherwise, do what `mod_req_idx_by` says
    pub fn scroll_down(&mut self) {
        self.mod_req_idx_by(1);
    }

    /// Select the previous request in the list, if it can. Otherwise, do what `mod_req_idx_by`
    /// says
    pub fn scroll_up(&mut self) {
        self.mod_req_idx_by(-1);
    }

    /// Get a list of the currently-viewable tabs
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Get a reference to the current tab, if one is currently selected that is not the main tab
    pub fn current_tab(&self) -> Option<&Tab> {
        self.current_tab_idx().and_then(|idx| self.tabs.get(idx))
    }

    /// Get a mutable reference to the current tab if one is selected that is not the main tab
    fn current_tab_mut(&mut self) -> Option<&mut Tab> {
        self.current_tab_idx()
            .and_then(|idx| self.tabs.get_mut(idx))
    }

    /// Get the current tab idx
    pub fn current_tab_idx(&self) -> Option<usize> {
        match self.current_pane {
            Pane::Tab { idx, .. } => Some(idx),
            Pane::Main { .. } => None,
        }
    }

    /// Push the currently selected request into a new tab at the end of the tab list
    pub async fn separate_current_req(&mut self) {
        // So we have to remove it first so that we can own it to clone it 'cause that's the
        // stupid workaround we have to deal with
        if let Some(req_idx) = self.current_req_idx.selected() {
            let req = self.requests.remove(req_idx);
            let (store, separate) = req.get_tab_copy().await;
            self.requests.insert(req_idx, store);

            self.tabs.push(Tab {
                req: separate,
                notes: String::new(),
                name: None,
            });
        }
    }

    /// Get the current pane
    pub fn current_pane(&self) -> &Pane {
        &self.current_pane
    }

    /// Try to gunzip the response on the current pane
    pub fn try_gunzip_current(&mut self) {
        if let Some(resp) = self
            .current_req_mut()
            .and_then(|r| r.resp.as_mut())
            .and_then(|r| r.response.as_mut().ok())
        {
            if let Err(e) = resp.try_gunzip() {
                self.show_error(format!(
                    "Can't gunzip response: {e}\nWas it already unzipped?"
                ));
            }
        }
    }

    /// Show the specified error string to the user
    pub fn show_error(&mut self, err: String) {
        self.err_msg = Some(err);
    }

    /// Pull up the system editor for the current request and save it back in
    pub fn edit_current_req_notes(&mut self) {
        if let Some(tab) = self.current_tab_mut() {
            match edit::edit(tab.notes.as_str()) {
                Ok(edited) => tab.notes = edited,
                Err(e) => self.show_error(format!("Couldn't edit notes: {e}")),
            }
        }
    }

    /// Rename the current tab
    pub fn rename_current_tab(&mut self, name: String) {
        if let Some(tab) = self.current_tab_mut() {
            tab.name = Some(name);
        }
    }
}

#[derive(Copy, Clone)]
pub enum PaneSelector {
    Idx(usize),
    Key(char),
}

impl LayoutState {
    /// Parse input text and use it to select a pane and/or tab
    pub fn select_pane_with_input(&mut self, input: PaneSelector) {
        // hmmm this would be better if we only tried to parse once we've verified that it doesn't
        // fit the other options but oh well, whatever
        match input {
            PaneSelector::Idx(i) => if i < self.tabs.len() {
                self.current_pane = Pane::Tab {
                    idx: i.min(self.tabs.len() - 1),
                    scroll: 0,
                    pane: TabPane::Notes,
                }
            }
            // use 'm' to select the main pane
            PaneSelector::Key('m') => {
                self.current_pane = Pane::Main {
                    scroll: 0,
                    pane: MainPane::ReqList,
                }
            }
            PaneSelector::Key(key) => match self.current_pane {
                Pane::Tab { idx, .. } => {
                    if let Ok(pane) = key.try_into() {
                        self.current_pane = Pane::Tab {
                            idx,
                            scroll: 0,
                            pane,
                        };
                    }
                }
                Pane::Main { .. } => {
                    if let Ok(pane) = key.try_into() {
                        self.current_pane = Pane::Main { scroll: 0, pane };
                    }
                }
            },
        }
    }
}

/// The information about every specific tab. The request assigned to it, the notes the user's
/// added to it, and the name (if the user has renamed the tab for ease of reference)
pub struct Tab {
    // cloned from the LayoutState.requests, not referencing it, cause this has to be held by
    // `LayoutState`, and if we tried to reference something in `requests`, it would be
    // self-referential, which we don't want to try
    pub req: Request,
    // just for now... i guess we could change it in the future
    pub notes: String,
    pub name: Option<String>,
}
