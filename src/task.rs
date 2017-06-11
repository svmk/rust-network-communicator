use curl::easy::Easy;
use std::boxed::Box;

/// Task for network manager.
pub enum Task<T> {
	Process {
		data: T,
		configurator: Box<Fn(&mut T,&mut Easy) + Send + 'static>,
	},
	Terminate,
}

impl <T>Task<T> {
	/// Creates new task with payload
	pub fn new<F>(payload: T,configurator: F) -> Task<T> where F: Send + 'static, F: Fn(&mut T,&mut Easy) {
		let task = Task::Process {
			data: payload,
			configurator: Box::new(configurator),
		};
		return task;
	}
}

pub fn disassemble_task<T>(task: Task<T>) -> (T,Easy) {
	match task {
		Task::Process { mut data, configurator } => {
			let mut request = Easy::new();
			configurator(&mut data,&mut request);
			return (data,request);
		},
		_ => {
			panic!("Unable to disassemble task");
		}
	}
}
pub fn generate_terminate_task<T>() -> Task<T> {
	Task::Terminate
}

pub fn is_terminate_task<T>(task: &Task<T>) -> bool {
	match task {
		&Task::Terminate => true,
		_ => false,
	}
}