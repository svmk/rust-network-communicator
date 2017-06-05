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

#[derive(Clone)]
pub struct RequestDownloader {
	inner: Option<Response>,
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
	pub fn new(task:Task,session: &Session,result_tx: SyncSender<RequestDownloaderResult>) -> Result<RequestDownloader,CurlError> {
		let inner = Some(
			Response{
				content: vec![],
			}
		);
		let mut curl_inner = inner.clone();
		let download_content = task.download();
		let mut curl_request = task.get_request();
		if download_content {
			let _ = curl_request.write_function(move |data| {
				curl_inner.as_mut().expect(
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