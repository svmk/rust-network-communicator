use tokio_core::reactor::{Core,Remote};
use std::thread::Builder as ThreadBuilder;
use std::thread::JoinHandle;
use std::sync::{Arc,Mutex,RwLock};
use tokio_curl::Session;
use std::sync::mpsc::{sync_channel,SyncSender,Receiver};
use tokio_core::reactor::Handle;
use futures::Future;
use futures::future;
use std::sync::mpsc::{RecvError,SendError};
use task::{is_terminate_task,generate_terminate_task};
use std::ops::Drop;
use std::mem::swap as swap_variables;
use request_future::RequestFuture;
use super::Task;
use super::RequestDownloader;
use super::RequestDownloaderResult;
use super::Error;
use super::Config;

struct Worker {
	remote: Remote,
	session: Arc<Mutex<Session>>,
	thread_handle: JoinHandle<()>,
	is_terminating: Arc<RwLock<bool>>,
}

impl Worker {
	pub fn new() -> Worker {
		let (tx,rx) = sync_channel::<(Arc<Mutex<Session>>,Remote)>(0);
		let is_terminating = Arc::new(RwLock::new(false));
		let is_terminating_thread = is_terminating.clone();
		let thread_handle = ThreadBuilder::new().spawn(
			move || {
				let mut lp = Core::new().expect("Unable to init downloader event-loop");
				let session = Arc::new(Mutex::new(Session::new(lp.handle())));
				let remote = lp.remote();
				tx.send((session,remote)).expect("Unable to send session and remote");
				loop {
					{
						let is_terminating = is_terminating_thread.read().expect("Unable to lock mutex");
						if *is_terminating {
							break;
						}
					}
					lp.turn(None);
				}
			}
		).expect(
			"Unable to init woker thread"
		);
		let (session,remote) = rx.recv().expect("Unablet to get session and remote");
		return Worker {
			remote: remote,
			session: session,
			thread_handle: thread_handle,
			is_terminating: is_terminating,
		};
	}

	fn terminate(self) {
		{
			let mut is_terminating = self.is_terminating.write().expect("Unable to lock mutex");
			*is_terminating = true;
		}
		self.remote.spawn(move |_handle:&Handle|{
			future::ok::<(),()>(())
		});
		self.thread_handle.join().expect("Unable to stop thread");

	}
}

/// Handle for working with network manager.
#[derive(Debug)]
pub struct NetworkManagerHandle<T: Send + 'static,E: Send + 'static> {
	task_rx: SyncSender<Task<T,E>>,
	result_tx: Receiver<RequestDownloaderResult<T,E>>,
	manager_handle: Option< JoinHandle<()> >,
}

impl <T: Send + 'static,E: Send + 'static>NetworkManagerHandle<T,E> {
	/// Aynchronous sending task to network manager.
	pub fn send(&self,task: Task<T,E>) -> Result<(), SendError<Task<T,E>>> {
		return self.task_rx.send(task);
	}

	/// Returns copy of task sender.
	pub fn get_sender(&self) -> SyncSender<Task<T,E>> {
		return self.task_rx.clone();
	}

	/// Receives result with locking.
	pub fn recv(&self) -> Result<RequestDownloaderResult<T,E>, RecvError> {
		return self.result_tx.recv();
	}
}

impl <T: Send + 'static,E: Send + 'static>Drop for NetworkManagerHandle<T,E> {
	/// When dropping we are waiting for termination of all threads.
	fn drop(&mut self) {
		self.task_rx.send(
			generate_terminate_task()
		).expect(
			"Unable to send termination task"
		);
		let mut manager_handle: Option<JoinHandle<()>> = None;
		swap_variables(&mut manager_handle,&mut self.manager_handle);
		manager_handle.unwrap().join().expect(
			"Unable to wait download manager thread"
		);
	}
}

/// Manager for processsing request.
pub struct NetworkManager<T: Send + 'static,E: Send + 'static> {
	remotes: Vec<Worker>,
	result_tx: SyncSender<RequestDownloaderResult<T,E>>,
}

impl <T: Send + 'static,E: Send + 'static>NetworkManager<T,E> {

	fn terminate_workers(&mut self) {
		for worker in self.remotes.drain(..) {
			worker.terminate();
		}
	}

	/// Creates new network manager.
	/// Produces threads that may panic when something is going wrong.
	pub fn start(config: &Config) -> Result<NetworkManagerHandle<T,E>,Error<E>> {
		let mut remotes = vec![];
		let (result_tx,result_rx) = sync_channel::<RequestDownloaderResult<T,E>>(config.get_limit_result_channel());
		for _ in 0..config.get_thread_count() {
			remotes.push(Worker::new());
		}
		let mut manager = NetworkManager {
			remotes: remotes,
			result_tx: result_tx.clone(),
		};
		let (tx,rx) = sync_channel::<Task<T,E>>(config.get_limit_task_channel());
		let thread_handle = ThreadBuilder::new().spawn(
			move || {
				for worker in manager.remotes.iter().cycle() {
					let task = rx.recv().expect("Unable to get task");
					if is_terminate_task(&task) {
						break;
					}
					let manager_result_tx = manager.result_tx.clone();
					let worker_session = worker.session.clone();
					worker.remote.spawn(move |_handle:&Handle|{
						let request_result = RequestDownloader::new(task,&*worker_session.lock().unwrap(),manager_result_tx.clone());
						let result = match request_result {
							Ok(request) => {
								RequestFuture::Process(request)
							},
							Err(request_error) => {
								manager_result_tx.send(
									Err( request_error )
								).expect("Unable to send result");
								RequestFuture::Ready
							},
						};
						return result.map(|_|{()}).map_err(|_|{()});
					});
				}
				manager.terminate_workers();
			}
		);
		match thread_handle {
			Ok(thread_handle) => {
				return Ok(NetworkManagerHandle {
					task_rx: tx,
					result_tx: result_rx,
					manager_handle: Some(thread_handle),
				});
			},
			Err(thread_error) => {
				return Err(Error::ThreadStartError { error: thread_error });
			}
		}
	}
}