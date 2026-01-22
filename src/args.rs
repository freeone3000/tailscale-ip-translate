use std::env;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub(crate) struct Args {
    pub(crate) reverse: bool,
    pub(crate) addr: String,
    pub(crate) translator_id: u32,
}

#[derive(Debug)]
pub(crate) struct ArgParseError {
    pub(crate) msg: String,
}
impl std::fmt::Display for ArgParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Argument parsing error: {}. Usage: <program> [--reverse] [translator_id] <address>", self.msg)
    }
}
impl std::error::Error for ArgParseError {
}

fn show_help(program_name: Option<String>) {
    let program_name = program_name.unwrap_or_else(|| "<program>".to_string());
    eprintln!("Converts between IPv4 and IPv6 addresses using a specific translation scheme.");
    eprintln!("Usage: {} [--reverse] [translator_id] <address>", program_name);
    eprintln!("  --reverse, -r       Convert from IPv6 to IPv4");
    eprintln!("  translator_id       Optional translator ID (default 7) for IPv4 to IPv6 conversion");
    eprintln!("  address             The IP address to convert");
}

pub(crate) fn parse() -> Result<Args, ArgParseError> {
    let mut args = Args::default();
    let mut arg_iter = env::args();
    let program_name = arg_iter.next(); // skip program name
    for arg in arg_iter {
        if arg == "--help" || arg == "-h" {
            show_help(program_name);
            std::process::exit(0);
        } else if arg == "--version" || arg == "-v" {
            println!("Version {}", VERSION);
            std::process::exit(0);
        } else if arg == "--reverse" || arg == "-r" {
            args.reverse = true;
        } else if args.addr.is_empty() {
            if arg.parse::<u32>().is_ok() {
                args.translator_id = arg.parse::<u32>().unwrap();
            } else {
                args.addr = arg;
            }
        } else {
            return Err(ArgParseError{msg: format!("Unknown argument: {}", arg)});
        }
    }
    if args.addr.is_empty() {
        return Err(ArgParseError{msg: "No address provided".to_string()});
    }

    Ok(args)
}