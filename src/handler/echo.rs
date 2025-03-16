use std::time::UNIX_EPOCH;

use evdev::EventType;
use evdev::InputEvent;
use evdev::uinput::VirtualDevice;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use super::EventHandler;
use crate::output::build_device;

pub struct EchoEventHandler {
    echo_stream: StandardStream,
    output_device: VirtualDevice,
}

impl EchoEventHandler {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let output_device =
            build_device().map_err(|e| format!("Failed to build an output device: {}", e))?;
        // Using Stderr for this.
        let echo_stream = StandardStream::stderr(ColorChoice::Always);
        Ok(Self {
            output_device,
            echo_stream,
        })
    }

    fn send_event(&mut self, event: InputEvent) -> std::io::Result<()> {
        self.output_device.emit(&[event])
    }
}

impl EventHandler for EchoEventHandler {
    fn handle_event(&mut self, event: InputEvent) -> Result<(), Box<dyn std::error::Error>> {
        if event.event_type() != EventType::KEY {
            self.send_event(event)?;
            return Ok(());
        }
        //println!()
        self.echo_stream
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        let timestamp = event.timestamp().duration_since(UNIX_EPOCH)?;
        if event.value() == 1 {
            println!(
                "Timestamp: {:>12}\t PRESS   \tKind: {:?}",
                timestamp.as_millis(),
                event.destructure()
            );
        } else if event.value() == 0 {
            println!(
                "Timestamp: {:>12}\t RELEASE \tKind: {:?}",
                timestamp.as_millis(),
                event.destructure()
            );
        } else {
            // ?
        }

        self.echo_stream.reset()?;
        self.send_event(event)?;
        Ok(())
    }
}
