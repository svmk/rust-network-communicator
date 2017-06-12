use curl::easy::Easy;
use std::boxed::Box;
use super::Error;

/// Task for network manager.
pub enum Task<T,E> {
	Process {
		data: T,
		configurator: Box<Fn(&mut T,&mut Easy) -> Result<(),Error<E>> + Send + 'static>,
	},
	Terminate,
}

impl <T,E>Task<T,E> {
	/// Creates new task with payload
	pub fn new<F>(payload: T,configurator: F) -> Task<T,E> where F: Send + 'static, F: Fn(&mut T,&mut Easy) -> Result<(),Error<E>> {
		let task = Task::Process {
			data: payload,
			configurator: Box::new(configurator),
		};
		return task;
	}
}

pub fn disassemble_task<T,E>(task: Task<T,E>) -> (T,Box<Fn(&mut T,&mut Easy) -> Result<(),Error<E>> + Send + 'static>) {
	match task {
		Task::Process { data, configurator } => {
			return (data, configurator);
		},
		_ => {
			panic!("Unable to disassemble task");
		}
	}
}
pub fn generate_terminate_task<T,E>() -> Task<T,E> {
	Task::Terminate
}

pub fn is_terminate_task<T,E>(task: &Task<T,E>) -> bool {
	match task {
		&Task::Terminate => true,
		_ => false,
	}
}