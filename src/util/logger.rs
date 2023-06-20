use env_logger::{Builder, Target};

pub fn initialize_logger() {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.filter(None, log::LevelFilter::Info);
    builder.init();
}
