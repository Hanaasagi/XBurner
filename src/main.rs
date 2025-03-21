mod config;
mod device;
mod executor;
mod handler;
mod x11;
use std::cmp::Ordering;

use env_logger;
mod input;
mod keycode;
mod notification;
mod output;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use clap::{Parser, Subcommand};
use evdev::Device as EDevice;

// Package meta info
const NAME: &str = env!("CARGO_PKG_NAME");
//const VERSION: &str = env!("CARGO_PKG_VERSION");
//const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
//const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Subcommand)]
enum Commands {
    // TODO install uninstall stop service edit ...
    /// Run the main application
    Run {
        /// Configuration file path
        #[arg(short, long)]
        config: String,

        /// Keyboard devices to grab
        #[arg(short, long)]
        device: String,
    },

    /// List devices information of this computer
    ListDevice {},

    /// List supported keys reported by the device
    ListKeys {
        /// Device path
        #[arg(short, long)]
        device: String,
    },

    /// Echo key information that you typed
    Echo {
        /// Keyboard devices to grab
        #[arg(short, long)]
        device: String,
    },
}

#[derive(Parser)]
#[command(about, version, author)]
#[command(arg_required_else_help = true)]
struct Args {
    /// Specify the subcommand to run
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Suppress output of all key events
    #[arg(long)]
    silent: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    match &args.command {
        Commands::ListDevice {} => {
            println!("Trying to scan all of {}", device::INPUT_DEVICE_PATH);
            let mut devices: Vec<(String, EDevice)> =
                device::DeviceManager::scan()?.into_iter().collect();
            devices.sort_by(|item_1, item_2| {
                let name_1 = &item_1.0;
                let name_2 = &item_2.0;
                match name_1.len().cmp(&name_2.len()) {
                    Ordering::Equal => name_1.cmp(&name_2),
                    other => other,
                }
            });
            println!("Available devices:");
            for (path, device) in devices.into_iter() {
                println!("{:20}: {}", path, device.name().unwrap_or("Unknown Name"));
            }
            return Ok(());
        }
        Commands::ListKeys { device } => {
            let device = device::DeviceManager::get_device(device)?;
            let keys = device
                .supported_keys()
                .expect("Could not get supported keys from this device");
            for key in keys.iter() {
                println!("{:?}", key);
            }
            return Ok(());
        }
        Commands::Echo { device } => {
            // TODO
            //let res = device::DeviceManager::scan()?;
            //let devices = res
            //    .into_values()
            //    .filter(|d| d.name().unwrap().contains("HHKB"))
            //    .collect::<Vec<EDevice>>();

            let device = device::DeviceManager::get_device(device)?;
            let event_handler = handler::EchoEventHandler::new()?;

            let term = Arc::new(AtomicBool::new(false));
            signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;
            let mut event_loop =
                input::EventLoop::new(vec![device], Box::new(event_handler), term)?;
            event_loop.run()?;
        }
        Commands::Run { config, device } => {
            // Load user config
            let config = config::Config::load_from_file(config)?;

            let device = device::DeviceManager::get_device(device)?;
            let event_handler = handler::DefaultEventHandler::new(&config)?;

            let term = Arc::new(AtomicBool::new(false));
            signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;
            let mut event_loop =
                input::EventLoop::new(vec![device], Box::new(event_handler), term)?;

            // Send start notification, silent if we meet error.
            notification::send_notify(
                NAME,
                &format!("{} is running now, your keyboard is grabbed.", NAME),
            )
            .ok();

            event_loop.run()?;

            notification::send_notify(NAME, &format!("{} is stopped now.", NAME)).ok();
        }
    }
    Ok(())
}
