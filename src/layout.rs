use crate::{
    input_state::InputState, request::Request, response_waiter::RequestResponse, ProxyInteraction,
};
use hyper::Body;
use ratatui::widgets::ListState;
use tokio::sync::oneshot;
use uuid::Uuid;

/// Records what the current state of the UI is, e.g. what tab the user is currently viewing,
/// what request is assigned to each tab, etc.
#[derive(Default)]
pub struct LayoutState {
    requests: Vec<Request>,
    current_req_idx: ListState,
    tabs: Vec<Tab>,
    // None for selected indicates Main tab, otherwise 0-indexed in `tabs`
    current_tab_idx: Option<usize>,
    pub input: InputState,
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
        self.current_req_idx
            .selected()
            .and_then(|idx| self.requests.get(idx))
    }

    /// Same, just if you want to do something to it
    pub fn current_req_mut(&mut self) -> Option<&mut Request> {
        self.current_req_idx
            .selected()
            .and_then(|idx| self.requests.get_mut(idx))
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
            println!("Couldn't find request for id {}", resp.id);
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
        if self.requests.is_empty() {
            return;
        }

        let selected = self.current_req_idx.selected().map_or(0, |sel| {
            (sel as isize + amt)
                .try_into()
                .unwrap_or(0)
                .max(self.requests.len() - 1)
        });

        self.current_req_idx.select(Some(selected));
    }

    /// Select the next request in the list, if it can. Otherwise, do what `mod_req_idx_by` says
    pub fn next_req(&mut self) {
        self.mod_req_idx_by(1);
    }

    /// Select the previous request in the list, if it can. Otherwise, do what `mod_req_idx_by`
    /// says
    pub fn prev_req(&mut self) {
        self.mod_req_idx_by(-1);
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn current_tab(&self) -> Option<&Tab> {
        self.current_tab_idx.and_then(|idx| self.tabs.get(idx))
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
