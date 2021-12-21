use std::collections::HashMap;
use std::error::Error;
use std::fs::read_dir;
use std::os::unix::ffi::OsStrExt;

use evdev::Device;

pub const INPUT_DEVICE_PATH: &str = "/dev/input";

pub struct DeviceManager {}

impl DeviceManager {
    pub fn scan() -> Result<HashMap<String, Device>, Box<dyn Error>> {
        let mut path_devices = HashMap::new();

        if let Some(dev_input) = read_dir(INPUT_DEVICE_PATH).as_mut().ok() {
            while let Some(entry) = dev_input.next() {
                let path = entry?.path();
                if let Some(fname) = path.file_name() {
                    if fname.as_bytes().starts_with(b"event") {
                        let device = Device::open(&path)?;
                        if let Ok(path) = path.into_os_string().into_string() {
                            path_devices.insert(path, device);
                        }
                    }
                }
            }
        }
        return Ok(path_devices);
    }

    pub fn get_device(path: &str) -> Result<Device, Box<dyn Error>> {
        let device = Device::open(path)?;
        Ok(device)
    }
}
