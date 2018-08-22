extern crate failure;
extern crate fern;
#[macro_use]
extern crate log;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate uploads_im_client;
extern crate webbrowser;

use failure::Error;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        if std::env::var("RUST_BACKTRACE").is_ok() {
            eprintln!("{}", e.backtrace());
        }
        for cause in e.iter_chain().skip(1) {
            eprintln!("caused by: {}", cause);
            if let Some(backtrace) = cause.backtrace() {
                eprintln!("backtrace: {}", backtrace);
            }
        }
    }
}

fn run() -> Result<(), Error> {
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    struct CommandLineOptions {
        #[structopt(short = "l", long = "verbosity", default_value = "warn")]
        log_level: log::LevelFilter,
        #[structopt(short = "i", long = "input")]
        upload_path_string: String,
    }

    let options = CommandLineOptions::from_args();

    fern::Dispatch::new()
        .level(options.log_level)
        .chain(std::io::stdout())
        .apply()?;

    let uploaded_image =
        uploads_im_client::upload_with_default_options(&options.upload_path_string)?;

    info!("uploaded_image: {:#?}", uploaded_image);

    webbrowser::open(uploaded_image.view_url.as_str())?;

    Ok(())
}
