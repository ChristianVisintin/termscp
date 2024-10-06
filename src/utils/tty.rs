//! ## Utils
//!
//! `Utils` implements utilities functions to work with layouts

use tuirealm::terminal::TerminalBridge;

/// Read a secret from tty with customisable prompt
pub fn read_secret_from_tty(
    terminal_bridge: &mut TerminalBridge,
    prompt: &str,
) -> std::io::Result<Option<String>> {
    let _ = terminal_bridge.disable_raw_mode();
    let _ = terminal_bridge.leave_alternate_screen();
    let res = match rpassword::prompt_password(prompt) {
        Ok(p) if p.is_empty() => Ok(None),
        Ok(p) => Ok(Some(p)),
        Err(err) => Err(err),
    };

    let _ = terminal_bridge.enter_alternate_screen();
    let _ = terminal_bridge.enable_raw_mode();

    res
}
