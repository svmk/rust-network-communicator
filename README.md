# rust-network-communicator
Network manager for download/upload content based on tokio-curl crate.

[![Build Status](https://travis-ci.org/svmk/rust-network-communicator.svg?branch=master)](https://travis-ci.org/svmk/rust-network-communicator)
[![Crates.io](https://img.shields.io/crates/v/rust-network-communicator.svg?maxAge=2592000)](https://crates.io/crates/rust-network-communicator)


[Documentation](https://docs.rs/network-communicator)
[Example](tests/download.rs)


# Usage
First, add this to your Cargo.toml:

```toml
[dependencies]
network-communicator = "0.1"
```
Next, add this to your crate:

```rust
extern crate network_communicator;
```
# License

tokio-curl is primarily distributed under the terms the MIT license.

See LICENSE for details.