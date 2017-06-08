use curl::easy::Easy;
use curl::Error as CurlError;
use std::mem::swap as swap_variable;

pub struct Task {
	download: bool,
	request: Option<Easy>,
}

impl Task {
	pub fn get(url:&str) -> Result<Task,CurlError> {
		let mut request = Easy::new();
		let _ = request.get(true)?;
		request.url(url)?;
		Ok(
			Task {
				download: true,
				request: Some(request),
			}
		)
	}
	pub fn configure(mut self,configurator: &Fn(Easy) -> Result<Easy,CurlError>) -> Result<Task,CurlError>  {
		self.request = Some(configurator(self.request.unwrap())?);
		return Ok(self);
	}
	pub fn get_request(&mut self) -> Easy {
		let mut request = None;
		swap_variable(&mut self.request,&mut request);
		return request.unwrap();
	}
	pub fn download(&self) -> bool {
		return self.download;
	}
}

pub fn generate_terminate_task() -> Task {
	Task {
		download: false,
		request: None,
	}
}

pub fn is_terminate_task(task: &Task) -> bool {
	return task.request.is_none();
}