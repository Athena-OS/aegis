use flexi_logger::{style, DeferredNow, LogSpecification, Logger};
use log::LevelFilter;
use std::io::Write;

pub fn init(verbosity: u8) {
    let log_specification = match verbosity {
        0 => LogSpecification::builder()
            .default(LevelFilter::Info)
            .build(),
        1 => LogSpecification::builder()
            .default(LevelFilter::Debug)
            .build(),
        _ => LogSpecification::builder()
            .default(LevelFilter::Trace)
            .build(),
    };
    Logger::with(log_specification)
        .format(format_log_entry)
        .start()
        .unwrap();
}

/// Formats a log entry with color
fn format_log_entry(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &log::Record,
) -> std::io::Result<()> {
    let msg = record.args().to_string();
    let level = record.level();
    
    // Get the current time as a NaiveTime
    let time = now.now().time();
    
    // Format the time as a string
    let time_str = time.format("%H:%M:%S").to_string();
    
    write!(
        w,
        "[ {} ] {} {}",
        style(level).paint(level.to_string()),
        time_str,
        msg
    )
}