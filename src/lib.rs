//! # Description
//! Creates thread pool for downloading content.
//! It's provides channel for sending requests and receiving results.
#[allow(deprecated)]
extern crate curl;
extern crate mio;
#[macro_use]
extern crate quick_error;
mod task;
mod error;
mod config;
mod event_loop;
mod manager;
pub use self::task::Task;
pub use self::error::Error;
pub use self::config::{Config,ConfigError};
pub use self::manager::NetworkManager;
pub use self::manager::NetworkManagerHandle;

#[test]
fn test_network_manager() {
	use curl::easy::Easy2;
	use curl::easy::Handler as EasyHandler;
	use curl::easy::WriteError;
	use std::io::Write;
	#[derive(Debug)]
	struct Payload {
		data: Vec<u8>,
	};
	impl Payload {
		pub fn new() -> Payload {
			Payload {
				data: Vec::new(),
			}
		}
	}
	impl EasyHandler for Payload {
		fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
            self.data.write(data).unwrap();
            Ok(data.len())
        }
	}
	let config = Config::new(2).expect("Unable to init config");
	let handle = NetworkManager::start::<Payload,()>(&config).expect(
		"Unable init network manager"
	);
	handle.send(Task::new(||{
		let mut url = Easy2::new(Payload::new());
		url.url("http://github.com")?;
		url.follow_location(true)?;
		Ok(url)
	})).expect("Unable to send task");
	let _ = handle.get_active_requests();
	let request = handle.recv().expect("Unable to recv request");
	assert_eq!(request.is_ok(),true);
	let mut request = request.unwrap();
	assert_eq!(request.response_code().unwrap(), 200);
	assert!(request.get_ref().data.len() >= 1);
}