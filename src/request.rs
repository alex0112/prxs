use crate::response_waiter::RequestResponse;
use hyper::{Body, Response};
use std::ops::Deref;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;

#[derive(Debug)]
pub enum RequestInteraction {
    r#Drop,
    Forward,
}

// we need the proxy to receive the interaction, so it knows what to do,
// and we also need it to receive another sender, to which it can send the result
// of trying to forward the message if it does need to forward it
pub enum ProxyInteraction {
    Drop,
    Forward(Sender<Result<Response<Body>, String>>),
}

#[derive(Debug)]
pub struct Request {
    pub id: Uuid,
    /// None once the interaction has been reported
    pub interaction_tx: Option<Sender<ProxyInteraction>>,
    pub inner: hyper::Request<Body>,
    pub resp: Option<RequestResponse>,
}

impl Request {
    #[must_use]
    pub async fn send_interaction(
        &mut self,
        interaction: RequestInteraction,
    ) -> Option<Receiver<Result<Response<Body>, String>>> {
        let Some(tx) = self.interaction_tx.take() else {
            return None;
        };

        match interaction {
            RequestInteraction::Drop => {
                if let Err(e) = tx.send(ProxyInteraction::Drop).await {
                    println!("Couldn't tell proxy to drop request: {e}");
                }
                None
            }
            RequestInteraction::Forward => {
                let (proxy_tx, proxy_rx) = channel(1);

                match tx.send(ProxyInteraction::Forward(proxy_tx)).await {
                    Err(e) => {
                        println!("Couldn't tell proxy to forward request: {e}");
                        None
                    }
                    Ok(_) => Some(proxy_rx),
                }
            }
        }
    }

    pub async fn store_response(&mut self, resp: RequestResponse) {
        self.resp = Some(resp);
    }
}

impl Deref for Request {
    type Target = hyper::Request<Body>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
