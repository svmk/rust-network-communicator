//! # Description
//! Creates thread pool for downloading content.
//! It's provides channel for sending requests and receiving results.
//!
//! # Example
//! 

//!```no_run
//! extern crate network_communicator;
//! extern crate curl;
//! use network_communicator::NetworkManagerHandle;
//! use network_communicator::NetworkManager;
//! use network_communicator::Config;
//! use network_communicator::Task;
//! use std::sync::{Arc,Mutex};
//! 
//! struct Payload {
//! 	input: Arc<Mutex<Vec<u8>>>,
//! }
//! 
//! impl Payload {
//! 	fn new() -> Payload {
//! 		Payload {
//! 			input: Arc::new(Mutex::new(vec![])),
//! 		}
//! 	}
//! }
//! 
//! fn network_manager() -> NetworkManagerHandle<Payload,()> {
//! 	let mut config = Config::new(1).expect("Unable to create config");
//! 	config.set_limit_task_channel(10000).unwrap();
//! 	config.set_limit_result_channel(10000).unwrap();
//! 	let manager = NetworkManager::start(&config).expect("Unable to create network manager");	
//! 	return manager;
//! }
//! 
//! fn get_request(url:&str) -> Task<Payload,()> {
//! 	let url = String::from(url);
//! 	Task::new(Payload::new(),move |payload,request|{
//! 		let payload_input = payload.input.clone();
//! 		request.url(&url)?;
//! 		request.write_function(move |data| {
//! 			payload_input.lock().unwrap().extend_from_slice(data);
//! 			Ok(data.len())
//! 		})?;
//! 		Ok(())
//! 	})
//! }
//! 
//! let manager = network_manager();
//! manager.send(get_request("https://github.com")).expect("Unable to send request");
//! manager.send(get_request("https://rust-lang.org")).expect("Unable to send request");
//!```
extern crate tokio_core;
extern crate tokio_curl;
extern crate futures;
extern crate curl;
#[macro_use]
extern crate quick_error;
mod request;
mod request_future;
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
