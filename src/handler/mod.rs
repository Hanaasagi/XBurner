mod default;
mod echo;

pub use default::*;
pub use echo::EchoEventHandler;
use evdev::InputEvent;

pub trait EventHandler {
    fn handle_event(&mut self, event: InputEvent) -> Result<(), Box<dyn std::error::Error>>;
}
