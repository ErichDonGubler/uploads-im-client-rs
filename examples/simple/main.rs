extern crate failure;
extern crate fern;
extern crate log;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate uploads_im_client;

use failure::Error;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
    }
}

fn run() -> Result<(), Error> {
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    struct CommandLineOptions {
        #[structopt(short = "i", long = "input")]
        upload_path_string: String
    }

    let options = CommandLineOptions::from_args();

    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;

    uploads_im_client::upload_with_default_options(&options.upload_path_string)
}
