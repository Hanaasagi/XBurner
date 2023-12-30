use std::error;

use x11rb;
use x11rb::properties::WmClass;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

#[allow(dead_code)]
pub struct X11Client {
    conn: RustConnection,
    screen_num: usize,
}

impl X11Client {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        let dpy_name: Option<&str> = None;
        let (conn, screen_num) = x11rb::connect(dpy_name)?;

        Ok(Self { conn, screen_num })
    }

    pub fn get_focus_window_wmclass(&self) -> Result<WmClass, Box<dyn error::Error>> {
        let res = self.conn.get_input_focus()?.reply()?;
        let window = res.focus;
        let wm_class = WmClass::get(&self.conn, window)?.reply()?;
        if wm_class.is_none() {
            return Err("No WM_CLASS".into());
        }
        Ok(wm_class.unwrap())
    }

    pub fn reconnect(&mut self) -> Result<(), Box<dyn error::Error>> {
        let dpy_name: Option<&str> = None;
        let (conn, screen_num) = x11rb::connect(dpy_name)?;
        self.conn = conn;
        self.screen_num = screen_num;

        Ok(())
    }
}
