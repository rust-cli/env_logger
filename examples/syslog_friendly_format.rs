fn main() {
    env_logger::builder()
        .parse_default_env()
        // While journald-logging is auto-detected, but you can manually override it.
        // Especially useful if you are using a different logging system.
        .format_syslog(true)
        .init();

    // Prints in a human readable way if run interactively,
    // and in a syslog-compatible way if run as a systemd service.
    log::info!("we are logging");
}
