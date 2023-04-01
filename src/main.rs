mod logger;

fn main() {
    logger::setup_logger(logger::LogLevel::Debug)
        .expect("unexpected error whie starting the logger");
}
