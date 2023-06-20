pub fn set_ctrlc_handler() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}
