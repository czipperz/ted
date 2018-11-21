use parking_lot::Mutex;

lazy_static! {
    static ref LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn log<'a, S>(s: S) where S: Into<String> {
    let mut log = LOG.lock();
    log.push(s.into());
}

pub fn print_log() {
    let log = LOG.lock();
    for l in log.iter() {
        println!("{}", l);
    }
}
