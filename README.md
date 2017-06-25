# rust-network-communicator
Network manager for download/upload content based on tokio-curl crate.

[![Build Status](https://travis-ci.org/svmk/rust-network-communicator.svg?branch=master)](https://travis-ci.org/svmk/rust-network-communicator)

[Documentation](https://docs.rs/tokio-core)
[Example](tests/download.rs)


# Usage
```toml
First, add this to your Cargo.toml:

[dependencies]
network-communicator = "0.1"
Next, add this to your crate:

extern crate network_communicator;
```

# License

tokio-curl is primarily distributed under the terms the MIT license.

See LICENSE for details.