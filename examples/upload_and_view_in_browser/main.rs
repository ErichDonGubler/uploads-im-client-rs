use {
    anyhow::{anyhow, Error},
    env_logger::init,
    itertools::Itertools,
    log::{error, info},
    reqwest::Client,
    std::env::args,
    tokio::task::spawn_blocking,
    uploads_im_client::upload_with_default_options,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    init();

    let (upload_path,) = args()
        .collect_tuple()
        .ok_or(anyhow!("expected upload path as a single arg"))?;

    let uploaded_image =
        upload_with_default_options(&mut Client::new(), upload_path.into()).await?;

    info!("uploaded_image: {:#?}", uploaded_image);

    let _ = spawn_blocking(move || {
        if let Err(e) = webbrowser::open(uploaded_image.view_url.as_str()) {
            error!("error opening web browser: {}", e);
        }
    })
    .await;

    Ok(())
}
