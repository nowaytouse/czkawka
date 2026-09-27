use log::error;

pub fn send_scan_completed_notification(tool: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        let tool = tool.to_owned();
        let body = body.to_owned();
        // The legacy macOS backend pumps the main run loop when its handle is dropped.
        if let Err(e) = spawn_notification(move || send_notification(&tool, &body)) {
            error!("Failed to start desktop notification thread: {e}");
        }
    }

    #[cfg(not(target_os = "macos"))]
    send_notification(tool, body);
}

#[cfg(target_os = "macos")]
fn spawn_notification(send: impl FnOnce() + Send + 'static) -> std::io::Result<()> {
    std::thread::Builder::new().name("krokiet-notification".into()).spawn(send).map(drop)
}

fn send_notification(tool: &str, body: &str) {
    #[cfg(target_os = "linux")]
    if try_notify_send(tool, body) {
        return;
    }

    // Fallback: notify-rust (covers macOS, Windows and Linux without notify-send)
    let mut notif = notify_rust::Notification::new();
    notif.summary(tool).body(body);
    #[cfg(all(unix, not(target_os = "macos")))]
    notif.urgency(notify_rust::Urgency::Normal);
    match notif.show() {
        Ok(handle) => {
            drop(handle);
            #[cfg(target_os = "macos")]
            log::info!("Desktop notification dispatch attempted");
            #[cfg(not(target_os = "macos"))]
            log::info!("Desktop notification sent");
        }
        Err(e) => error!("Failed to send desktop notification: {e}"),
    }
}

#[cfg(target_os = "linux")]
// TODO - linux on error - https://github.com/hoodie/notify-rust/issues/218
fn try_notify_send(summary: &str, body: &str) -> bool {
    match std::process::Command::new("notify-send").arg("--app-name=krokiet").arg(summary).arg(body).status() {
        Ok(s) if s.success() => {
            log::info!("Desktop notification sent via notify-send");
            true
        }
        Err(e) => {
            error!("Failed to execute notify-send: {e}");
            false
        }
        Ok(failed) => {
            error!("notify-send exited with non-zero status: {failed}");
            false
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use super::spawn_notification;

    #[test]
    fn notification_runs_outside_caller_thread() {
        let caller_id = thread::current().id();
        let (sender, receiver) = mpsc::channel();
        spawn_notification(move || {
            let worker = thread::current();
            sender.send((worker.id(), worker.name().map(str::to_owned))).unwrap();
        })
        .unwrap();

        let (worker_id, worker_name) = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_ne!(caller_id, worker_id);
        assert_eq!(worker_name.as_deref(), Some("krokiet-notification"));
    }
}
