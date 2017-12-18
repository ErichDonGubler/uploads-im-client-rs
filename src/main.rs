extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;

// mod UploadsIm {
//     #[derive(Debug, Clone, Deserialize)]
//     struct UploadResponse {
//         #[serde(rename = "img_name")]
//         image_name: String,
//         #[serde(rename = "img_url")]
//         image_url: Url,
//         #[serde(rename = "img_view")]
//         image_view_url: Url,
//         #[serde(rename = "img_attr")]
//         image_attributes: Url,
//         #[serde(rename = "img_view")]
//         image_size: ,
//         #[serde(rename = "img_view")]
//         image_view_url: Url,
//     }
// }

fn main() {
    let form = reqwest::multipart::Form::new().file("fileupload", "./sample.jpg").expect("successful form build");

    let mut upload = reqwest::Client::new()
        .post("http://uploads.im/api?upload")
        .multipart(form)
        .send()
        .expect("successful upload");

    println!("upload: {:#?}", upload);
    println!("upload.text: {}", upload.text().expect("upload response text"));
}
