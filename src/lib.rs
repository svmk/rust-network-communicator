extern crate tokio_core;
extern crate tokio_curl;
extern crate futures;
extern crate curl;
#[macro_use]
extern crate quick_error;
mod request;
mod manager;
mod task;
mod config;
mod error;
use self::request::RequestDownloader;
pub use self::request::RequestDownloaderResult;
pub use self::task::Task;
pub use self::manager::NetworkManager;
pub use self::manager::NetworkManagerHandle;
pub use self::error::Error;
pub use self::config::{Config,ConfigError};