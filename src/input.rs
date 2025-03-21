use std::error::Error;
use std::os::fd::BorrowedFd;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use evdev::Device;
use log::info;
use nix::errno::Errno;
use nix::sys::select::FdSet;
use nix::sys::select::select;

use super::handler::EventHandler;

/// Main EventLoop, receive device events and call the event_handler to process them.
pub struct EventLoop<'a> {
    /// List of devices to listen to
    input_devices: Vec<Device>,
    /// Callback handler
    event_handler: Box<dyn EventHandler + 'a>,
    /// Stop Flag
    stop_flag: Arc<AtomicBool>,
    // TODO Reload?
}

impl<'a> EventLoop<'a> {
    pub fn new(
        input_devices: Vec<Device>,
        event_handler: Box<dyn EventHandler + 'a>,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            input_devices,
            event_handler,
            stop_flag,
        })
    }

    fn grab_devices(&mut self) -> Result<(), Box<dyn Error>> {
        for device in self.input_devices.iter_mut() {
            device.grab()?
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.grab_devices()?;

        let mut read_fds = FdSet::new();
        for device in self.input_devices.iter() {
            let raw_fd = device.as_raw_fd();
            // let fd = Box::new(unsafe { BorrowedFd::borrow_raw(raw_fd) });
            // read_fds.insert(Box::leak(fd));
            read_fds.insert(unsafe { BorrowedFd::borrow_raw(raw_fd) });
        }

        loop {
            // `select` is a slow syscall, it will return when we receive a signal.
            // If error is `EINTR`, we need to retry.
            let res = select(None, &mut read_fds, None, None, None);
            if let Some(err) = res.err() {
                if err == Errno::EINTR {
                    continue;
                }
            }
            // let select_res = self.select_readable_devices();

            // Check if it needs to stop, the current implementation is very simple.
            // Known problems is that we need to press any key to stop after SIGINT.
            // There are many ways to do this:
            // 1. A thread to check signal and write a char to pipe fd,
            // and put this fd to `select`
            // 2. ...
            if self.stop_flag.load(Ordering::Relaxed) {
                info!("Stop now...");
                return Ok(());
            }

            let readable_fds = read_fds;
            for input_device in self.input_devices.iter_mut() {
                if !readable_fds
                    .contains(unsafe { BorrowedFd::borrow_raw(input_device.as_raw_fd()) })
                {
                    continue;
                }
                for event in input_device.fetch_events()? {
                    self.event_handler.handle_event(event)?;
                }
            }
        }
    }
}
