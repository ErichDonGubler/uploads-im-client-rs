# Uploads.im Client

[![Linux build status](https://travis-ci.org/ErichDonGubler/uploads-im-client-rs.svg)](https://travis-ci.org/ErichDonGubler/uploads-im-client-rs)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/ErichDonGubler/uploads-im-client-rs?svg=true)](https://ci.appveyor.com/project/ErichDonGubler/uploads-im-client-rs)
[![crates.io latest published version](https://img.shields.io/crates/v/uploads-im-client.svg)](https://crates.io/crates/uploads-im-client)
[![docs.rs latest published version](https://docs.rs/uploads-im-client/badge.svg)](https://docs.rs/uploads-im-client)
[![Apache License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE-2.0.md)
[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE-MIT.md)

Bindings to the [Uploads.im](http://uploads.im/) [web API](http://uploads.im/apidocs).
Upload your images for free with Rust!

## Overview

The Uploads.im API currently has only the `upload` endpoint, which allows anyone
to upload an image file with no authentication. Here's an example of how to use
it:

```rust
extern crate uploads_im_client;

fn main() {
    let uploaded_image = uploads_im_client::upload_with_default_options("my_image.jpg").expect("successful image upload");
    println!("Uploaded image! You can now view it at {}", uploaded_image.view_url.to_string());
}
```

## Licensing

This project is dual-licensed under either the MIT or Apache 2.0 license. Take
your pick!

## Contributing

Contributions, feature requests, and bug reports are warmly welcomed! See the
[contribution guidelines](./CONTRIBUTING.md) for getting started.

See also the [Code of Conduct](./CODE-OF-CONDUCT.md) for more details about
expectations regarding contributions to this project.

## Contributors

@ErichDonGubler, original author
