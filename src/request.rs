use hyper::Body;
use std::ops::Deref;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub enum RequestInteraction {
    r#Drop,
    Forward,
}

#[derive(Debug)]
pub struct Request {
    /// None once the interaction has been reported
    pub interaction_tx: Option<Sender<RequestInteraction>>,
    pub inner: hyper::Request<Body>,
}

impl Request {
    pub async fn send_interaction(&mut self, interaction: RequestInteraction) {
        if let Some(tx) = self.interaction_tx.take() {
            if let Err(e) = tx.send(interaction).await {
                println!("Couldn't tell proxy to interact with request: {e}");
            }
        }
    }
}

impl Deref for Request {
    type Target = hyper::Request<Body>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
