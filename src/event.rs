use crossterm::event::Event;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event receiver channel.
    receiver: UnboundedReceiver<std::io::Result<Event>>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new() -> Self {
        let (input_tx, input_rx) = unbounded_channel();
        std::thread::spawn(move || loop {
            input_tx
                .send(crossterm::event::read())
                .unwrap_or_else(|e| panic!("Couldn't send event to Tui: {e}"));
        });

        Self { receiver: input_rx }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> std::io::Result<Event> {
        self.receiver
            .recv()
            .await
            .expect("The channel will never be explicitly closed")
    }
}
