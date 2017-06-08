use curl::Error as CurlError;
use curl::easy::Easy;
use futures::Future;
use tokio_curl::Session;
use tokio_curl::Perform;
use futures::Poll;
use tokio_curl::PerformError;
use futures::Async;
use super::Task;
use super::Error;
use std::error::Error as StdError;
use std::sync::mpsc::SyncSender;
use task::{get_data_from_task,get_request_from_task};


pub struct RequestDownloader<T: Send + 'static> {
	task: Option<Task<T>>,
	request: Perform,
	sender: SyncSender<RequestDownloaderResult<T>>,
}

impl <T: Send + 'static>Future for RequestDownloader<T> {
	type Item = Easy;
    type Error = PerformError;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    	let result = self.request.poll();
    	match result {
    		Ok(Async::Ready(_)) => {
				let response = get_data_from_task(self.task.as_mut().unwrap());
				self.sender.send(Ok(response)).expect("Unable to send response");
    		},
    		Ok(Async::NotReady) => {},
    		Err(ref error) => {
    			self.sender.send(
	    			Err(Error::EventLoop (
	    				String::from(error.description()),
	    				format!("{:?}",error)
	    			))
    			).expect("Unable to send response error");
    		}
    	}
    	return result;
    }
}


/// Result of working network manager.
pub type RequestDownloaderResult<T> = Result<T,Error>;

impl <T: Send + 'static>RequestDownloader<T> {
	pub fn new(mut task:Task<T>,session: &Session,result_tx: SyncSender<RequestDownloaderResult<T>>) -> Result<RequestDownloader<T>,CurlError> {
		let downloader = RequestDownloader {
			request: session.perform(get_request_from_task(&mut task)),
			task: Some(task),
			sender: result_tx,
		};
		return Ok(downloader);
	}
}