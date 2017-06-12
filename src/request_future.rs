use curl::easy::Easy;
use request::RequestDownloader;
use futures::Poll;
use futures::Async;
use futures::Future;
use tokio_curl::PerformError;

pub enum RequestFuture<T: Send + 'static,E: Send + 'static> {
	Process(RequestDownloader<T,E>),
	Ready,
}

pub enum RequestFutureItem {
	Process(Easy),
	Ready,
}


impl <T: Send + 'static,E: Send + 'static>Future for RequestFuture<T,E> {
	type Item = RequestFutureItem;
    type Error = PerformError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    	match self {
    		&mut RequestFuture::Process(ref mut result) => {
    			return result.poll().map(
    				|result|{
    					return result.map(|item|{
    						RequestFutureItem::Process(item)
    					})
    				}
				);
    		},
			&mut RequestFuture::Ready => {
				return Ok(Async::Ready(RequestFutureItem::Ready));
			}
    	}
    }
}