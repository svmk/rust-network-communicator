use curl::easy::Easy;
use curl::Error as CurlError;

pub struct Task {
	download: bool,
	request: Easy,
}

impl Task {
	pub fn get(url:&str) -> Result<Task,CurlError> {
		let mut request = Easy::new();
		let _ = request.get(true)?;
		request.url(url)?;
		Ok(
			Task {
				download: true,
				request: request,
			}
		)
	}
	pub fn configure(mut self,configurator: &Fn(Easy) -> Result<Easy,CurlError>) -> Result<Task,CurlError>  {
		self.request = configurator(self.request)?;
		return Ok(self);
	}
	pub fn get_request(self) -> Easy {
		return self.request;
	}
	pub fn download(&self) -> bool {
		return self.download;
	}
}