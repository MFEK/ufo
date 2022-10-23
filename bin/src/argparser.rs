// Argument parser
use git_version::git_version;

use clap; //argparse lib

pub struct Args {
    pub filename: Option<String>,
}

pub fn parse_args() -> Args {
    let matches = clap::App::new("MFEKufo")
        .version(&*format!("{}-alpha", git_version!()))
        .about("Font viewer, Modular Font Editor K project")
        .arg(
            clap::Arg::with_name("UFO")
                .help("Input UFO directory")
                .index(1),
        )
        .get_matches();
    Args {
        filename: matches.value_of("UFO").map(|s| s.to_string()),
    }
}
