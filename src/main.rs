extern crate abletime;
extern crate clap;

use clap::Clap;

const ABLETON_SUFFIX: &str = ".als";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clap, Debug)]
#[clap(version = VERSION)]
struct Opts {
    /// Directory to inspect. Defaults to current directory
    #[clap(default_value = ".")]
    directory: String,

    /// Project file suffix. Default value works for Ableton projects
    #[clap(short, long, default_value = ABLETON_SUFFIX)]
    suffix: String,

    /// Maximum number of minutes allowed between saves for time to be counted.
    /// Values <= 0 will disable this feature
    #[clap(short, long, default_value = "60")]
    max_minutes_between_saves: i64,
}

fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();

    let project_files = match abletime::scan_project_files(opts.directory, opts.suffix, opts.max_minutes_between_saves)
    {
        Ok(project_files) => project_files,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1)
        }
    };
    abletime::print_project_summary(&project_files);

    Ok(())
}
