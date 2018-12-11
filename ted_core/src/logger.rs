use parking_lot::Mutex;

lazy_static! {
    static ref LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

/// Log a message
///
/// This is similar to `println!`.  The message is placed into the log
/// and is visible to the user at a later point.
pub fn log<S>(s: S)
where
    S: Into<String>,
{
    let mut log = LOG.lock();
    log.push(s.into());
}

/// Log a message
///
/// This is similar to `println!`.  The message is placed into the log
/// and is visible to the user at a later point.
///
/// If the program is compiled in debug mode, this message will be
/// logged.
pub fn log_debug<S>(s: S)
where
    S: Into<String>,
{
    if cfg!(debug_assertions) {
        let mut log = LOG.lock();
        log.push(s.into());
    }
}

/// Print the log
///
/// This will probably be replaced soon with a buffer that stores the
/// log.
pub fn print_log() {
    let log = LOG.lock();
    for l in log.iter() {
        println!("{}", l);
    }
}
