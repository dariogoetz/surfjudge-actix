use slog::Logger;
use sloggers::terminal::TerminalLoggerBuilder;
use sloggers::Build;

use once_cell::sync::Lazy;

pub static LOG: Lazy<Logger> = Lazy::new(|| {
    let builder = TerminalLoggerBuilder::new();
    builder.build().unwrap()
});
