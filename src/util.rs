use colored::Colorize;

pub fn hard_error(msg: &str) -> ! {
    eprintln!("{}", msg.bright_red());
    std::process::exit(1)
}
