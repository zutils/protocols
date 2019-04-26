use std::sync::Mutex;
use std::thread::{self, ThreadId};
use hashbrown::HashMap;

lazy_static::lazy_static! {
    static ref TAB_HASH: Mutex<HashMap<ThreadId, usize>> = { std::sync::Mutex::new(HashMap::new()) };
}

pub fn initialize_standard_logging(log_prefix: &'static str) -> Result<(), failure::Error> {
    use fern::colors::{Color, ColoredLevelConfig};

    let mut colors = ColoredLevelConfig::new();
    colors.error = Color::BrightRed;
    colors.warn = Color::BrightYellow;
    colors.info = Color::BrightGreen;
    colors.debug = Color::BrightMagenta;
    colors.trace = Color::BrightBlue;

    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(move |out, message, record| {
            let message = format!("{}", message);
            let mut tab_hash = TAB_HASH.lock().unwrap();
            let tab_count: &mut usize = tab_hash.entry(thread::current().id()).or_default();
            
            // Remove the tabs prior to printing
            if message.starts_with("...") {
                *tab_count-=1;
            }

            let tabs: String = std::iter::repeat("| ").take(*tab_count).collect();
            let formatted = format!("{:?}\t{}{}", thread::current().id(), tabs, log_prefix);

            match record.level() {
                log::Level::Info => out.finish(format_args!("{}{}{}", formatted, colors.color(record.level()), message)),
                log::Level::Debug => out.finish(format_args!("{}{}", formatted, message)),
                log::Level::Trace => out.finish(format_args!("{}{}{}", formatted, record.target(), message)),
                _ => out.finish(format_args!("{}{}{}", formatted, colors.color(record.level()), message)),
            }

            // Insert the tabs after printing
            if message.ends_with("...") {
                *tab_count+=1;
            }
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("mio", log::LevelFilter::Info)
        .level_for("tokio_reactor", log::LevelFilter::Info)
        .level_for("tokio_threadpool", log::LevelFilter::Info)
        .level_for("reqwest", log::LevelFilter::Info)
        .level_for("wasmer_runtime_core", log::LevelFilter::Info)
        .level_for("wasmer_runtime", log::LevelFilter::Info)
        .level_for("wasmer_wasi", log::LevelFilter::Info)
        .level_for("wasmer", log::LevelFilter::Info)
        .level_for("wabt", log::LevelFilter::Info)
        .level_for("cranelift_codegen", log::LevelFilter::Info)
        .level_for("cranelift_wasm", log::LevelFilter::Warn)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        // Apply globally
        .apply()?;

    log::trace!("Logging initialized!");
    Ok(())
}
