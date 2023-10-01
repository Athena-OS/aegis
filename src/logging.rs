use flexi_logger::{style, DeferredNow, FileSpec, LogSpecification, Logger, Duplicate};
use log::LevelFilter;
use std::fs;
use std::io::Write;
use crate::internal::files;

pub fn init(verbosity: u8, log_file_path: &str) {
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

    // Check if the log file already exists
    if fs::metadata(log_file_path).is_ok() {
        // If an old log file exists, remove it
        files::remove_file(log_file_path);
    }
    
    // Create a file-based logger and specify the log file path
    Logger::with(log_specification)
        .log_to_file(
            FileSpec::default()
                .basename(log_file_path)
                .suffix("log") // log file extension. Not using .log because does not apply ANSI color code
                .suppress_timestamp(),
        )
        .duplicate_to_stderr(Duplicate::All) // Duplicate logs to stderr for console output
        .format(format_log_entry)
        .start() // It will create the new log file
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
