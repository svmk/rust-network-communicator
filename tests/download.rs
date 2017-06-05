extern crate network_communicator;
use network_communicator::NetworkManagerHandle;
use network_communicator::NetworkManager;
use network_communicator::Config;
use network_communicator::Task;

fn network_manager() -> NetworkManagerHandle {
	let mut config = Config::new(1).expect("Unable to create config");
	config.set_limit_task_channel(10000).unwrap();
	config.set_limit_result_channel(10000).unwrap();
	let manager = NetworkManager::start(&config).expect("Unable to create network manager");	
	return manager;
}

#[test]
pub fn test_drop_network_manager_1() {
	let manager = network_manager();
	manager.send(Task::get("https://github.com").unwrap()).expect("Unable to send request");
	manager.send(Task::get("https://rust-lang.org").unwrap()).expect("Unable to send request");
}

#[test]
pub fn test_drop_network_manager_2() {
	let manager = network_manager();
	manager.send(Task::get("https://github.com").unwrap()).expect("Unable to send request");
	let result_1 = manager.recv();
	assert!(result_1.is_ok());
	assert!(result_1.unwrap().is_ok());
	manager.send(Task::get("https://rust-lang.org").unwrap()).expect("Unable to send request");
	let result_2 = manager.recv();
	assert!(result_2.is_ok());
	assert!(result_2.unwrap().is_ok());
}