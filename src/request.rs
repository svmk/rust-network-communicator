use curl::Error as CurlError;
use curl::easy::Easy;
use futures::Future;
use tokio_curl::Session;
use tokio_curl::Perform;
use futures::Poll;
use tokio_curl::PerformError;
use curl::easy::WriteError;
use std::sync::Arc;
use futures::Async;
use std::mem::swap as swap_variable;
use super::Task;
use super::Error;
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

pub struct RequestDownloader {
	inner: Option<Response>,
	request: Perform,
	sender: SyncSender<RequestDownloaderResult>,
}

impl Future for RequestDownloader {
	type Item = Easy;
    type Error = PerformError;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    	let result = self.request.poll();
    	match result {
    		Ok(Async::Ready(_)) => {
    			let mut response = None;
				swap_variable(&mut response,&mut self.inner);
				self.sender.send(Ok(response.unwrap())).expect("Unable to send response result");
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
	pub fn new(mut task:Task,session: &Session,result_tx: SyncSender<RequestDownloaderResult>) -> Result<RequestDownloader,CurlError> {
		let inner = Some(
			Response{
				content: vec![],
			}
		);
		let mut curl_inner = inner.clone();
		let download_content = task.download();

		let downloader = RequestDownloader {
			inner: inner,
			request: session.perform(task.get_request()),
			sender: result_tx,
		};
		return Ok(downloader);
	}
}