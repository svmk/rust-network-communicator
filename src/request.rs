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
use task::{disassemble_task};
use std::mem::swap as swap_variable;


pub struct RequestDownloader<T: Send + 'static> {
	payload: Option<T>,
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
    			let mut response = None;
    			swap_variable(&mut self.payload, &mut response);
				self.sender.send(Ok(response.unwrap())).expect("Unable to send response");
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
	pub fn new(task:Task<T>,session: &Session,result_tx: SyncSender<RequestDownloaderResult<T>>) -> Result<RequestDownloader<T>,CurlError> {
		let (payload,request) = disassemble_task(task);
		let downloader = RequestDownloader {
			request: session.perform(request),
			payload: Some(payload),
			sender: result_tx,
		};
		return Ok(downloader);
	}
}