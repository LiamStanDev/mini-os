use log::{Level, LevelFilter, Log};

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true // show all levels
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let color = match record.level() {
            Level::Error => 31, // Red
            Level::Warn => 93,  // BrightYellow
            Level::Info => 34,  // Blue
            Level::Debug => 32, // Green
            Level::Trace => 90, // BrightBlack
        };

        let display_level = match record.level() {
            Level::Error => "ERR", // Red
            Level::Warn => "WRN",  // BrightYellow
            Level::Info => "INF",  // Blue
            Level::Debug => "DBG", // Green
            Level::Trace => "TRC", // BrightBlack
        };

        println!(
            "\u{1B}[{}m[{}] {}\u{1B}[0m",
            color,
            display_level,
            record.args()
        )
    }

    fn flush(&self) {}
}

pub(crate) fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });
}
