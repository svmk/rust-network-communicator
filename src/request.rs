use curl::Error as CurlError;
use curl::easy::Easy;
use futures::Future;
use tokio_curl::Session;
use tokio_curl::Perform;
use futures::Poll;
use tokio_curl::PerformError;
use curl::easy::WriteError;
use std::sync::{Arc,Mutex};
use futures::Async;
use super::Task;
use super::Error;
use std::cell::Cell;
use std::error::Error as StdError;
use std::sync::mpsc::SyncSender;


#[derive(Debug,Clone)]
pub struct Response {
	pub content: Vec<u8>,
}

impl Response {
	fn write(&mut self,data: &[u8]) -> Result<usize,WriteError> {
		self.content.extend_from_slice(data);
		Ok(data.len())
	}
}

#[derive(Clone)]
pub struct RequestDownloader {
	inner: Arc<Mutex<Cell<Option<Response>>>>,
	request: Arc<Perform>,
	sender: SyncSender<RequestDownloaderResult>,
}

impl Future for RequestDownloader {
	type Item = Easy;
    type Error = PerformError;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    	let result = Arc::get_mut(&mut self.request).unwrap().poll();
    	match result {
    		Ok(Async::Ready(_)) => {
    			let response = self.inner.lock().expect(
					"Unable to lock mutex"
				).replace(None).expect(
					"Unable to get response"
				);
				self.sender.send(Ok(response)).expect("Unable to send response result");
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
pub type RequestDownloaderResult = Result<Response,Error>;

impl RequestDownloader {
	pub fn new(task:Task,session: &Session,result_tx: SyncSender<RequestDownloaderResult>) -> Result<RequestDownloader,CurlError> {
		let inner = Arc::new(
			Mutex::new(
				Cell::new(
					Some(
						Response{
							content: vec![],
						}
					)
				)
			)
		);
		let curl_inner = inner.clone();
		let download_content = task.download();
		let mut curl_request = task.get_request();
		if download_content {
			let _ = curl_request.write_function(move |data| {
				curl_inner.lock().expect(
					"Unable to unwrap curl_inner"
				).get_mut().as_mut().expect(
					"Unable to write recv data"
				).write(data)
			})?;
		}

		let downloader = RequestDownloader {
			inner: inner,
			request: Arc::new(session.perform(curl_request)),
			sender: result_tx,
		};
		return Ok(downloader);
	}
}