#![deny(missing_docs)]
//! # Uploads.im Client
//!
//! This crate is a thin wrapper that models the Uploads.im web API. Currently,
//! the only functionality available to Uploads.im users is the `upload`
//! endpoint.
//!
//! NOTE: At the time of writing, the Uploads.im service is not accepting new
//! uploads. You will most likely need to find an alternate provider right now.
//!
//! # Examples
//!
//! ```rust,no_run
//! # #![allow(clippy::needless_doctest_main)]
//! use {reqwest::Client, uploads_im_client::upload_with_default_options};
//!
//! #[tokio::main]
//! async fn main() {
//!     let uploaded_image = upload_with_default_options(
//!         &mut Client::new(),
//!         "my_image.jpg".to_owned().into(),
//!     ) .await.expect("successful image upload");
//!     println!("Uploaded image! You can now view it at {}", uploaded_image.view_url.as_str());
//! }
//! ```

use {
    derive_builder::Builder,
    log::{debug, info, trace},
    reqwest::{
        multipart::{Form, Part},
        Client, StatusCode,
    },
    serde::{
        de::{Error as DeserializationError, Unexpected},
        Deserialize, Deserializer,
    },
    std::{convert::TryFrom, path::PathBuf},
    thiserror::Error,
    url::Url,
};

/// The default host that the Uploads.im service uses in production.
pub const DEFAULT_HOST: &str = "uploads.im";

/// The integral type that thumbnail image dimensions use.
pub type ThumbnailDimension = u32;
/// The integral type that full image dimensions use.
pub type FullSizeDimension = u64;

/// Models options exposed to users of the upload API.
#[derive(Builder, Clone, Debug)]
pub struct UploadOptions {
    /// The domain hosting the Uploads.im service
    pub host: String,
    /// An optional width to which an uploaded image should be resized to.
    pub resize_width: Option<FullSizeDimension>,
    /// An optional width to which the thumbnail of an uploaded image should be
    /// resized to.
    pub thumbnail_width: Option<ThumbnailDimension>,
    /// An optional flag to mark an uploaded image as "family unsafe", or in
    /// other words, adult content or NSFW.
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

/// An abstract struct that encapsulates an image entry on the Uploads.im site.
#[derive(Debug, Clone)]
pub struct ImageReference<Dimension> {
    /// The dimensions of the referred image.
    pub dimensions: Rectangle<Dimension>,
    /// The URL through which the referred image can be requested.
    pub url: Url,
}

/// Represents a completed image upload to Uploads.im.
#[derive(Debug, Clone)]
pub struct UploadedImage {
    /// The name of an uploaded image. This usually does **not** match the name
    /// of the original uploaded image file. This name is usually an ID value,
    /// followed by the original extension of the uploaded image. For example,
    /// `something.jpg` may be renamed to `vwk7b.jpg`.
    pub name: String,
    /// A reference to the full-size uploaded image.
    pub full_size: ImageReference<FullSizeDimension>,
    /// A URL to a human-friendly page showing the uploaded image.
    pub view_url: Url,
    /// A reference to a thumbnail version of the uploaded image.
    pub thumbnail: ImageReference<ThumbnailDimension>,
    /// Flags whether or not the uploaded image was resized upon upload.
    pub was_resized: bool,
}

/// An abstract struct that represents a rectangular area.
#[derive(Debug, Clone)]
pub struct Rectangle<T> {
    /// The height of the rectangle
    height: T,
    /// The width of the rectangle
    width: T,
}

/// Represents the possible responses given the upload API.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawUploadResponse {
    /// Represents a upload failure
    Failure {
        #[serde(deserialize_with = "parse_status_code_string")]
        status_code: StatusCode,
        status_txt: String,
    },
    /// Represents an upload success
    Success {
        /// The data given in response to a successful image upload.
        data: Box<RawUploadResponseSuccess>,
    },
}

/// Deserializes an integral number string into an HTTP status code.
fn parse_status_code_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<StatusCode, D::Error> {
    let status_code_number = u16::deserialize(deserializer)?;
    StatusCode::from_u16(status_code_number).map_err(|_| {
        D::Error::invalid_value(
            Unexpected::Unsigned(u64::from(status_code_number)),
            &"valid HTTP status code",
        )
    })
}

/// Represents a success response for an image uploaded using the upload API.
#[derive(Debug, Clone, Deserialize)]
struct RawUploadResponseSuccess {
    img_name: String,
    img_url: Url,
    img_view: Url,
    #[serde(deserialize_with = "parse_u64_string")]
    img_height: FullSizeDimension,
    #[serde(deserialize_with = "parse_u64_string")]
    img_width: FullSizeDimension,
    thumb_url: Url,
    thumb_height: ThumbnailDimension,
    thumb_width: ThumbnailDimension,
    #[serde(deserialize_with = "parse_bool_number_string")]
    resized: bool,
}

/// Deserializes an integral string into a `u64`.
fn parse_u64_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    use std::error::Error as StdError;
    use std::num::ParseIntError;

    let string_value = String::deserialize(deserializer)?;
    Ok(string_value.parse().map_err(|e: ParseIntError| {
        let unexpected = Unexpected::Str(&string_value);
        D::Error::invalid_value(unexpected, &e.description())
    }))?
}

/// Deserializes an integral string into a `bool`.
fn parse_bool_number_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let parsed_number = parse_u64_string(deserializer)?;
    Ok(match parsed_number {
        0 => false,
        1 => true,
        _ => {
            let unexpected = Unexpected::Unsigned(parsed_number);
            return Err(D::Error::invalid_value(
                unexpected,
                &"boolean integral value",
            ));
        }
    })
}

impl TryFrom<RawUploadResponse> for UploadedImage {
    type Error = UploadError;
    fn try_from(response: RawUploadResponse) -> Result<Self, Self::Error> {
        match response {
            RawUploadResponse::Failure {
                status_code,
                status_txt,
            } => Err(UploadError::ResponseReturnedFailure {
                status_code,
                status_text: status_txt,
            }),
            RawUploadResponse::Success { data } => {
                let d = *data;
                let RawUploadResponseSuccess {
                    img_name,
                    img_url,
                    img_view,
                    img_height,
                    img_width,
                    thumb_url,
                    thumb_height,
                    thumb_width,
                    resized,
                } = d;

                Ok(UploadedImage {
                    name: img_name,
                    full_size: ImageReference {
                        url: img_url,
                        dimensions: Rectangle {
                            height: img_height,
                            width: img_width,
                        },
                    },
                    thumbnail: ImageReference {
                        url: thumb_url,
                        dimensions: Rectangle {
                            height: thumb_height,
                            width: thumb_width,
                        },
                    },
                    view_url: img_view,
                    was_resized: resized,
                })
            }
        }
    }
}

/// Represents an error that can occur when building an upload API URL.
#[derive(Debug, Error)]
pub enum UploadRequestURLBuildError {
    /// Indicates that the upload URL could not be built.
    #[error("URL params serialization failed")]
    URLParamsBuildingFailed(#[source] serde_urlencoded::ser::Error),
    /// Indicates that the built URL failed validation.
    #[error("URL validation failed")]
    URLValidationFailed(#[source] url::ParseError),
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

/// Represents an error that may occur when building and sending an image
/// upload request.
#[derive(Debug, Error)]
pub enum UploadError {
    /// Indicates a failure building an upload endpoint URL.
    #[error("failed building upload request")]
    BuildingRequest(
        #[from]
        #[source]
        UploadRequestURLBuildError,
    ),
    /// Indicates that the provided filename was invalid.
    #[error("invalid filename \"{}\"", _0.display())]
    InvalidFilename(PathBuf),
    /// Indicates a upload request transmission error.
    #[error("could not transmit upload request")]
    SendingRequest(
        #[from]
        #[source]
        reqwest::Error,
    ),
    /// Indicates an error response returned by the upload API.
    #[error(
        "the server returned HTTP error code {} (\"{}\")",
        status_code,
        status_text
    )]
    ResponseReturnedFailure {
        /// The status code returned by the server. Note that this code is
        /// contained in the *body* of the response, and not the header.
        status_code: StatusCode,
        /// A string describing the error returned by the API.
        status_text: String,
    },
    /// Indicates an error accessing a file for upload.
    #[error("cannot access file to upload")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
    /// Indicates an error parsing the response from the upload API.
    #[error("internal error: unable to parse upload response")]
    ParsingResponse(
        #[from]
        #[source]
        serde_json::Error,
    ),
}

/// Builds an upload endpoint URL given some `UploadOptions` suitable for a
/// multipart form upload to Uploads.im.
pub fn build_upload_url(options: &UploadOptions) -> Result<Url, UploadRequestURLBuildError> {
    let url_string = {
        let params = {
            let &UploadOptions {
                ref resize_width,
                ref family_unsafe,
                ..
            } = options;

            macro_rules! generate_string_keyed_pairs {
                ($($arg: tt),*) => { [$(generate_string_keyed_pairs!(@inside $arg)),*] };
                (@inside $e: ident) => { (stringify!($e), $e.map(|x| x.to_string())) };
                (@inside $e: expr) => { $e };
            }

            let params_tuple = generate_string_keyed_pairs![
                resize_width,
                family_unsafe,
                (
                    "thumb_width",
                    options.thumbnail_width.map(|x| x.to_string())
                )
            ];

            serde_urlencoded::to_string(params_tuple)?
        };
        let initial_params_separator = if params.is_empty() { "" } else { "&" };

        format!(
            "http://{}/api?upload{}{}",
            options.host, initial_params_separator, params
        )
    };

    Ok(Url::parse(&url_string)?)
}

/// Uploads an image file denoted by `file_path` using the given `options` to
/// the Uploads.im image upload API.
pub async fn upload(
    client: &mut Client,
    file_path: PathBuf,
    options: &UploadOptions,
) -> Result<UploadedImage, UploadError> {
    info!(
        "Beginning upload of file \"{}\" with {:#?}",
        file_path.display(),
        options
    );

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str().map(ToOwned::to_owned))
        .ok_or_else(|| UploadError::InvalidFilename(file_path))?;

    let endpoint_url = build_upload_url(options)?;

    debug!("Upload URL: {}", endpoint_url.as_str());
    let form = Form::new().part("fileupload", Part::stream("asdf").file_name(file_name));

    trace!("Request built, sending now...");

    let response = client
        .post(endpoint_url.as_str())
        .multipart(form)
        .send()
        .await?;

    debug!("Got upload response: {:#?}", response);

    let response_body_text = response.text().await?;

    debug!("Upload response data: {:#?}", response_body_text);

    let raw_upload_response: RawUploadResponse = serde_json::from_str(&response_body_text)?;

    debug!("Parsed response: {:#?}", raw_upload_response);

    let uploaded_image = UploadedImage::try_from(raw_upload_response)?;

    Ok(uploaded_image)
}

/// Uploads an image file denoted by `file_path` using default `options` to
/// the Uploads.im image upload API.
pub async fn upload_with_default_options(
    client: &mut Client,
    file_path: PathBuf,
) -> Result<UploadedImage, UploadError> {
    upload(client, file_path, &UploadOptions::default()).await
}
