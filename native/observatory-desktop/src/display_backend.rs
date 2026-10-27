//! Select the native display that can enforce passive notification policy.
//!
//! Winit defaults to Wayland whenever both sockets exist. Its xdg_toplevel
//! backend cannot express "never accept keyboard focus". XWayland is a native
//! display path and supports the ICCCM/EWMH hints used by passive_windows.
use x11rb::{
    connection::Connection, protocol::xproto::ConnectionExt, rust_connection::RustConnection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    X11,
    Wayland,
}

pub fn select(x11_reachable: bool) -> Backend {
    if x11_reachable {
        Backend::X11
    } else {
        Backend::Wayland
    }
}

/// Probe the advertised display; a stale DISPLAY must not prevent startup on
/// an otherwise usable Wayland session. This performs no focus mutation.
pub fn detect() -> Backend {
    let reachable = RustConnection::connect(None).is_ok_and(|(connection, _)| {
        connection
            .get_input_focus()
            .ok()
            .is_some_and(|request| request.reply().is_ok())
            && connection.flush().is_ok()
    });
    select(reachable)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn usable_xwayland_wins_over_default_wayland_backend() {
        assert_eq!(select(true), Backend::X11);
        assert_eq!(select(false), Backend::Wayland);
    }
}
