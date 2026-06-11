fn main() {
    use clap::Parser;

    let cli = godot_harbor_lib::cli::Cli::parse();

    if let Err(e) = godot_harbor_lib::cli::run(cli) {
        eprintln!("{} {}", console::style("错误:").red().bold(), e);
        std::process::exit(1);
    }
}
