// extern crate network_communicator;
// extern crate curl;
// use network_communicator::NetworkManagerHandle;
// use network_communicator::NetworkManager;
// use network_communicator::Config;
// use network_communicator::Task;
// use std::sync::{Arc,Mutex};

// struct Payload {
// 	input: Arc<Mutex<Vec<u8>>>,
// }

// impl Payload {
// 	fn new() -> Payload {
// 		Payload {
// 			input: Arc::new(Mutex::new(vec![])),
// 		}
// 	}
// }

// fn network_manager() -> NetworkManagerHandle<Payload,()> {
// 	let mut config = Config::new(1).expect("Unable to create config");
// 	config.set_limit_task_channel(10000).unwrap();
// 	config.set_limit_result_channel(10000).unwrap();
// 	let manager = NetworkManager::start(&config).expect("Unable to create network manager");	
// 	return manager;
// }

// fn get_request(url:&str) -> Task<Payload,()> {
// 	let url = String::from(url);
// 	Task::new(Payload::new(),move |payload,request|{
// 		let payload_input = payload.input.clone();
// 		request.url(&url)?;
// 		request.write_function(move |data| {
// 			payload_input.lock().unwrap().extend_from_slice(data);
// 			Ok(data.len())
// 		})?;
// 		Ok(())
// 	})
// }

// #[test]
// pub fn test_drop_network_manager_1() {
// 	let manager = network_manager();
// 	manager.send(get_request("https://github.com")).expect("Unable to send request");
// 	manager.send(get_request("https://rust-lang.org")).expect("Unable to send request");
// }

// #[test]
// pub fn test_drop_network_manager_2() {
// 	let manager = network_manager();
// 	manager.send(get_request("https://github.com")).expect("Unable to send request");
// 	let result_1 = manager.recv();
// 	assert!(result_1.is_ok());
// 	assert!(result_1.unwrap().is_ok());
// 	manager.send(get_request("https://google.com")).expect("Unable to send request");
// 	let result_2 = manager.recv();
// 	assert!(result_2.is_ok());
// 	assert!(result_2.unwrap().is_ok());
// }

// #[test]
// pub fn test_drop_network_manager_3() {
// 	let manager = network_manager();
// 	manager.send(get_request("https://github.com")).expect("Unable to send request");
// 	manager.send(get_request("https://google.com")).expect("Unable to send request");
// 	let result_1 = manager.recv();
// 	assert!(result_1.is_ok());
// 	assert!(result_1.unwrap().is_ok());
// 	let result_2 = manager.recv();
// 	assert!(result_2.is_ok());
// 	assert!(result_2.unwrap().is_ok());
// }