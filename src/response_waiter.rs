use crate::ConsumingClone;
use async_trait::async_trait;
use futures_core::stream::Stream;
use hyper::{Body, Response};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use uuid::Uuid;

#[derive(Debug)]
pub struct RequestResponse {
    pub id: Uuid,
    pub response: Result<Response<Body>, String>,
}

#[async_trait]
impl ConsumingClone for RequestResponse {
    async fn clone(self) -> (Self, Self) {
        let Self { id, response } = self;
        let (r1, r2) = match response {
            Err(e) => (Err(e.clone()), Err(e)),
            Ok(r) => {
                let (r1, r2) = r.clone().await;
                (Ok(r1), Ok(r2))
            }
        };

        (Self { id, response: r1 }, Self { id, response: r2 })
    }
}

type ResponseFut = Pin<Box<dyn Future<Output = RequestResponse>>>;

#[derive(Default)]
pub struct ResponseWaiter {
    requests: Vec<ResponseFut>,
    waker: Option<Waker>,
}

impl std::fmt::Debug for ResponseWaiter {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ResponseWaiter")
            .field("requests_len", &self.requests.len())
            .field("waker", &self.waker)
            .finish()
    }
}

impl ResponseWaiter {
    pub fn submit(&mut self, fut: ResponseFut) {
        let repoll = self.requests.is_empty();

        self.requests.push(fut);

        // We need to do this to tell the runtime to poll us again if we already exhausted all
        // child futures that could be the ones to tell the runtime to wake
        if repoll {
            // theoretically, we could not have a waker at this point, and thus we wouldn't call
            // wake, but since this is polled regularly with `select` in the App, I think we can
            // safely trust that a waker will be set and this should work fine. should.
            if let Some(waker) = self.waker.as_ref() {
                waker.wake_by_ref();
            }
        }
    }
}

impl Stream for ResponseWaiter {
    type Item = RequestResponse;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.waker = Some(cx.waker().clone());

        // check every one, in order of submission, to see if they're done
        // if none are, just return pending.
        // also if there are none stored, this will immediately return pending, and
        // self.waker.wake_by_ref() will be called when a new request gets submitted
        self.requests
            .iter_mut()
            .enumerate()
            .find_map(|(idx, fut)| match fut.as_mut().poll(cx) {
                Poll::Ready(resp) => Some((idx, resp)),
                Poll::Pending => None,
            })
            .map_or(Poll::Pending, |(idx, r)| {
                // Make sure to remove it from the list so we don't re-poll it
                // after it's completed
                self.requests.remove(idx);
                Poll::Ready(Some(r))
            })
    }
}
