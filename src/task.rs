use curl::easy::Easy;
use std::mem::swap as swap_variable;

/// Task for network manager.
pub struct Task<T> {
	data: Option<T>,
	request: Option<Easy>,
}

/// Function for request configuration.
pub type TaskConfigurator<T,E> = Fn(&mut T,&mut Easy) -> Result<(),E>;

impl <T>Task<T> {
	/// Creates new task with payload
	pub fn new<E>(payload: T,configurator: &TaskConfigurator<T,E>) -> Result<Task<T>,E> {
		let mut task = Task {
			request: Some(Easy::new()),
			data: Some(payload),
		};
		task.configure(configurator)?;
		Ok(
			task
		)
	}
	fn configure<E>(&mut self,configurator: &TaskConfigurator<T,E>) -> Result<(),E>  {
		return configurator(self.data.as_mut().unwrap(),self.request.as_mut().unwrap());
	}
}

pub fn get_request_from_task<T>(task: &mut Task<T>) -> Easy {
	let mut request = None;
	swap_variable(&mut task.request,&mut request);
	return request.unwrap();
}

pub fn get_data_from_task<T>(task: &mut Task<T>) -> T {
	let mut data = None;
	swap_variable(&mut task.data,&mut data);
	return data.unwrap();
}
pub fn generate_terminate_task<T>() -> Task<T> {
	Task {
		request: None,
		data: None,
	}
}

pub fn is_terminate_task<T>(task: &Task<T>) -> bool {
	return task.request.is_none();
}