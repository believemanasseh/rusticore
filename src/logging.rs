use log::{error, info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

/// Initialises the logging configuration for the server.
///
/// # Arguments
///
/// * `log_output` - An optional string slice specifying the log output destination.
/// * `debug` - A boolean indicating whether debug mode is enabled.
///
/// # Returns
///
/// `true` if the logging configuration was successfully initialised, `false` otherwise.
///
/// # Examples
///
/// ```
/// use log::{info, error};
/// use rusticore::init_logging;
///
/// let success = init_logging(Some("logs/app.log"), true);
/// if success {
///     info!("Logging initialised successfully.");
/// } else {
///     error!("Failed to initialise logging.");
/// }
/// ```
pub fn init_logging(log_output: Option<&'static str>, debug: bool) -> bool {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build();

    let mut appenders = vec![Appender::builder().build("stdout", Box::new(stdout))];
    let mut root_appenders = vec!["stdout"];

    if let Some(ref path) = log_output {
        let file = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build(path)
            .unwrap();
        appenders.push(Appender::builder().build("core", Box::new(file)));
        root_appenders.push("core");
    }

    let config = Config::builder()
        .appenders(appenders)
        .logger(
            Logger::builder()
                .additive(false)
                .appenders(root_appenders.to_owned())
                .build("app::core", LevelFilter::Info),
        )
        .logger(Logger::builder().build("app::none", LevelFilter::Off))
        .build(
            Root::builder()
                .appender(root_appenders[0])
                .build(LevelFilter::Info),
        );

    if let Ok(cfg) = config {
        match log4rs::init_config(cfg) {
            Ok(_) => {
                let target = if debug { "app::core" } else { "app::none" };
                info!(target: target, "Logging configuration initialised successfully.");
                true
            }
            Err(e) => {
                error!("Failed to initialise logging configuration: {e}");
                false
            }
        }
    } else {
        error!("Failed to create logging configuration.");
        false
    }
}
