#[macro_use]
extern crate derive_builder;
extern crate failure;
#[macro_use]
extern crate log;
extern crate reqwest;

use failure::Error;
use std::path::Path;

pub const DEFAULT_HOST: &str = "uploads.im";

#[derive(Builder, Clone, Debug)]
pub struct UploadOptions {
    pub host: String,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_owned()
        }
    }
}

pub fn upload_with_default_options<P: AsRef<Path>>(file_path: P) -> Result<(), Error> {
    upload(file_path, &UploadOptions::default())
}

pub fn upload<P: AsRef<Path>>(file_path: P, options: &UploadOptions) -> Result<(), Error> {
    let host = format!("http://{}/api?upload", options.host);

    info!("Beginning upload of file \"{}\" with {:#?}", file_path.as_ref().display(), options);

    let form = reqwest::multipart::Form::new().file("fileupload", file_path)?;

    let mut upload = reqwest::Client::new()
        .post(&host)
        .multipart(form)
        .send()?;

    debug!("Got upload response: {:#?}", upload);

    Ok(())
}
