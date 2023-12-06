use futures_core::stream::Stream;
use http::{HeaderMap, HeaderValue, StatusCode, Version};
use hyper::{body::Bytes, Body, Response};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RequestResponse {
    pub id: Uuid,
    pub response: Result<CopyableResponse, String>,
}

// We do this because we want to be able to display the `Bytes` of the body in the way we want.
// We can't do that if we just use `hyper::Response::Body` because you have to consume it to get
// the whole underlying bytes, which we can't do every time we want to display this
#[derive(Clone, Debug)]
pub struct CopyableResponse {
    pub body: Bytes,
    pub status: StatusCode,
    pub version: Version,
    pub headers: HeaderMap<HeaderValue>,
}

impl CopyableResponse {
    pub async fn from_resp(resp: Response<Body>) -> Self {
        let (parts, body) = resp.into_parts();
        let bytes = hyper::body::to_bytes(body)
            .await
            .expect("This body was created by hyper so it's trustworty");

        Self {
            body: bytes,
            status: parts.status,
            version: parts.version,
            headers: parts.headers,
        }
    }

    pub fn try_gunzip(&mut self) {
        if let Ok(res) = crate::gunzip(&self.body) {
            self.body = res.into();
        }
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
