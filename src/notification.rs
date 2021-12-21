use notify_rust::Notification;

use super::NAME;

pub fn send_notify(title: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .appname(NAME)
        .summary(title)
        .body(content)
        .timeout(2000)
        .show()?;
    Ok(())
}
