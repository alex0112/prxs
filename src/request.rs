use crate::{response_waiter::RequestResponse, ConsumingClone};
use hyper::{Body, Response};
use std::ops::Deref;
use tokio::sync::oneshot::{self, channel};
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub enum RequestInteraction {
    r#Drop,
    Forward,
}

// we need the proxy to receive the interaction, so it knows what to do,
// and we also need it to receive another sender, to which it can send the result
// of trying to forward the message if it does need to forward it
#[derive(Debug)]
pub enum ProxyInteraction {
    Drop,
    Forward(oneshot::Sender<Result<Response<Body>, String>>),
}

#[derive(Debug)]
pub struct Request {
    pub id: Uuid,
    /// None once the interaction has been reported
    pub interaction_tx: Option<oneshot::Sender<ProxyInteraction>>,
    pub inner: hyper::Request<Body>,
    pub resp: Option<RequestResponse>,
}

impl Request {
    #[must_use]
    pub fn send_interaction(
        &mut self,
        interaction: RequestInteraction,
    ) -> Option<oneshot::Receiver<Result<Response<Body>, String>>> {
        let Some(tx) = self.interaction_tx.take() else {
            return None;
        };

        match interaction {
            RequestInteraction::Drop => {
                if tx.send(ProxyInteraction::Drop).is_err() {
                    println!("Couldn't tell proxy to drop request {self:?}");
                }
                None
            }
            RequestInteraction::Forward => {
                let (proxy_tx, proxy_rx) = channel();

                match tx.send(ProxyInteraction::Forward(proxy_tx)) {
                    Err(_) => {
                        println!("Couldn't tell proxy to forward request {self:?}");
                        None
                    }
                    Ok(()) => Some(proxy_rx),
                }
            }
        }
    }

    pub fn store_response(&mut self, resp: RequestResponse) {
        self.resp = Some(resp);
    }

    pub async fn get_tab_copy(self) -> (Self, Self) {
        let Request {
            id,
            interaction_tx,
            inner,
            resp,
        } = self;
        let (inner1, inner2) = inner.clone().await;
        let resp_clone = resp.clone();

        (
            Self {
                id,
                interaction_tx,
                inner: inner1,
                resp,
            },
            Self {
                id,
                interaction_tx: None,
                inner: inner2,
                resp: resp_clone,
            },
        )
    }
}

impl Deref for Request {
    type Target = hyper::Request<Body>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
