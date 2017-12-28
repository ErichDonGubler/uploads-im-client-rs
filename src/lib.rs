#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate try_from;
extern crate url;
extern crate url_serde;

use reqwest::StatusCode;
use serde::de::{Error as DeserializationError, Unexpected};
use serde::{Deserialize, Deserializer};
use std::path::Path;
use try_from::TryFrom;
use url::Url;

pub const DEFAULT_HOST: &str = "uploads.im";

pub type ThumbnailDimension = u32;
pub type FullSizeDimension = u64;

#[derive(Builder, Clone, Debug)]
pub struct UploadOptions {
    pub host: String,
    pub resize_width: Option<FullSizeDimension>,
    pub thumbnail_width: Option<ThumbnailDimension>,
    pub family_unsafe: Option<bool>,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_owned(),
            resize_width: None,
            thumbnail_width: None,
            family_unsafe: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageReference<Dimension> {
    pub dimensions: ImageDimension<Dimension>,
    pub url: Url
}

#[derive(Debug, Clone)]
pub struct UploadedImage {
    pub name: String,
    pub full_size: ImageReference<FullSizeDimension>,
    pub view_url: Url,
    pub thumbnail: ImageReference<ThumbnailDimension>,
    pub was_resized: bool,
}

#[derive(Debug, Clone)]
pub struct ImageDimension<T> {
    height: T,
    width: T
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawUploadResponse {
    Failure {
        #[serde(deserialize_with = "parse_status_code_string")]
        status_code: StatusCode,
        status_txt: String
    },
    Success {
        data: RawUploadResponseData
    },
}

fn parse_status_code_string<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<StatusCode, D::Error> {
    let status_code_number = u16::deserialize(deserializer)?;
    StatusCode::try_from(status_code_number as u16)
        .map_err(|_| {
            D::Error::invalid_value(
                Unexpected::Unsigned(status_code_number as u64),
                &"valid HTTP status code"
            )
        })
}

#[derive(Debug, Clone, Deserialize)]
struct RawUploadResponseData {
    img_name: String,
    #[serde(with = "url_serde")]
    img_url: Url,
    #[serde(with = "url_serde")]
    img_view: Url,
    #[serde(deserialize_with = "parse_u64_string")]
    img_height: FullSizeDimension,
    #[serde(deserialize_with = "parse_u64_string")]
    img_width: FullSizeDimension,
    #[serde(with = "url_serde")]
    thumb_url: Url,
    thumb_height: ThumbnailDimension,
    thumb_width: ThumbnailDimension,
    #[serde(deserialize_with = "parse_bool_number_string")]
    resized: bool,
}

fn parse_u64_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    use serde::Deserialize;
    use std::error::Error as StdError;
    use std::num::ParseIntError;

    let string_value = String::deserialize(deserializer)?;
    Ok(string_value.parse().map_err(|e: ParseIntError| {
        let unexpected = Unexpected::Str(&string_value);
        D::Error::invalid_value(unexpected, &e.description())
    }))?
}

fn parse_bool_number_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let parsed_number = parse_u64_string(deserializer)?;
    Ok(match parsed_number {
        0 => false,
        1 => true,
        _ => {
            let unexpected = Unexpected::Unsigned(parsed_number);
            Err(D::Error::invalid_value(unexpected, &"boolean integral value"))?
        }
    })
}

impl TryFrom<RawUploadResponse> for UploadedImage {
    type Err = UploadError;
    fn try_from(response: RawUploadResponse) -> Result<Self, Self::Err> {
        match response {
            RawUploadResponse::Failure {
                status_code,
                status_txt,
            } => {
                Err(UploadError::ResponseReturnedFailure {
                    status_code,
                    status_text: status_txt,
                })
            },
            RawUploadResponse::Success {
                data: RawUploadResponseData {
                    img_name,
                    img_url,
                    img_view,
                    img_height,
                    img_width,
                    thumb_url,
                    thumb_height,
                    thumb_width,
                    resized,
                }
            } => Ok(UploadedImage {
                name: img_name,
                full_size: ImageReference {
                    url: img_url,
                    dimensions: ImageDimension {
                        height: img_height,
                        width: img_width
                    }
                },
                thumbnail: ImageReference {
                    url: thumb_url,
                    dimensions: ImageDimension {
                        height: thumb_height,
                        width: thumb_width
                    }
                },
                view_url: img_view,
                was_resized: resized,
            })
        }
    }
}

#[derive(Debug, Fail)]
pub enum UploadRequestURLBuildError {
    #[fail(display = "URL params serialization failed")]
    URLParamsBuildingFailed(#[cause] serde_urlencoded::ser::Error),
    #[fail(display = "URL validation failed")]
    URLValidationFailed(#[cause] url::ParseError),
}

impl From<url::ParseError> for UploadRequestURLBuildError {
    fn from(e: url::ParseError) -> Self {
        UploadRequestURLBuildError::URLValidationFailed(e)
    }
}

impl From<serde_urlencoded::ser::Error> for UploadRequestURLBuildError {
    fn from(e: serde_urlencoded::ser::Error) -> Self {
        UploadRequestURLBuildError::URLParamsBuildingFailed(e)
    }
}

#[derive(Debug, Fail)]
pub enum UploadError {
    #[fail(display = "internal error: failed building upload request")]
    BuildingRequest(#[cause] UploadRequestURLBuildError),
    #[fail(display = "could not transmit upload request")]
    SendingRequest(#[cause] reqwest::Error),
    #[fail(display = "the server returned HTTP error code {} (\"{}\")", status_code, status_text)]
    ResponseReturnedFailure {
        status_code: reqwest::StatusCode,
        status_text: String,
    },
    #[fail(display = "cannot access file to upload")]
    Io(#[cause] std::io::Error),
    #[fail(display = "internal error: unable to parse upload response")]
    ParsingResponse(#[cause] serde_json::Error),
}

impl From<UploadRequestURLBuildError> for UploadError {
    fn from(e: UploadRequestURLBuildError) -> Self {
        UploadError::BuildingRequest(e)
    }
}

impl From<reqwest::Error> for UploadError {
    fn from(e: reqwest::Error) -> Self {
        UploadError::SendingRequest(e)
    }
}

impl From<std::io::Error> for UploadError {
    fn from(e: std::io::Error) -> Self {
        UploadError::Io(e)
    }
}

impl From<serde_json::Error> for UploadError {
    fn from(e: serde_json::Error) -> Self {
        UploadError::ParsingResponse(e)
    }
}

pub fn build_upload_url(options: &UploadOptions) -> Result<Url, UploadRequestURLBuildError> {
    let url_string = {
        let params = {
            let &UploadOptions {
                ref resize_width,
                ref family_unsafe,
                ..
            } = options;

            use std::string::ToString;
            macro_rules! generate_string_keyed_pairs {
                ($($arg: tt),*) => { [$(generate_string_keyed_pairs!(@inside $arg)),*] };
                (@inside $e: ident) => { (stringify!($e), $e.map(|x| x.to_string())) };
                (@inside $e: expr) => { $e };
            }

            let params_tuple = generate_string_keyed_pairs![
                resize_width,
                family_unsafe,
                ("thumb_width", options.thumbnail_width.map(|x| x.to_string()))
            ];

            serde_urlencoded::to_string(params_tuple)?
        };
        let initial_params_separator = if params.is_empty() { "" } else { "&" };

        format!("http://{}/api?upload{}{}", options.host, initial_params_separator, params)
    };

    Ok(Url::parse(&url_string)?)
}

pub fn upload<P: AsRef<Path>>(file_path: P, options: &UploadOptions) -> Result<UploadedImage, UploadError> {
    info!("Beginning upload of file \"{}\" with {:#?}", file_path.as_ref().display(), options);

    let endpoint_url = build_upload_url(options)?;

    debug!("Upload URL: {}", endpoint_url.as_str());

    let form = reqwest::multipart::Form::new().file("fileupload", file_path)?;

    trace!("Request built, sending now...");

    let mut response = reqwest::Client::new()
        .post(endpoint_url.as_str())
        .multipart(form)
        .send()?;

    debug!("Got upload response: {:#?}", response);

    let response_body_text = response.text()?;

    debug!("Upload response data: {:#?}", response_body_text);

    let raw_upload_response: RawUploadResponse = serde_json::from_str(&response_body_text)?;

    debug!("Parsed response: {:#?}", raw_upload_response);

    let uploaded_image = UploadedImage::try_from(raw_upload_response)?;

    Ok(uploaded_image)
}

pub fn upload_with_default_options<P: AsRef<Path>>(file_path: P) -> Result<UploadedImage, UploadError> {
    upload(file_path, &UploadOptions::default())
}
