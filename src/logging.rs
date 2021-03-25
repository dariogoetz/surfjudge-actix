use slog::Logger;
use sloggers::terminal::TerminalLoggerBuilder;
use sloggers::Build;
use sloggers::types::Severity;

use once_cell::sync::Lazy;

pub static LOG: Lazy<Logger> = Lazy::new(|| {
    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.build().unwrap()
});
