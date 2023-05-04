use kallax_cli::CommandError;

fn main() {
    if let Err(err) = kallax_cli::Cli::default().run() {
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }
}
