use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Refresh,
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    pub async fn next(&self) -> Result<AppEvent> {
        let timeout = self.tick_rate;

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) => Ok(AppEvent::Key(key_event)),
                Event::Resize(width, height) => Ok(AppEvent::Resize(width, height)),
                _ => Ok(AppEvent::Refresh),
            }
        } else {
            Ok(AppEvent::Refresh)
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(Duration::from_millis(250))
    }
}
