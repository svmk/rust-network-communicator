use tokio_core::reactor::{Core,Remote};
use std::thread::Builder as ThreadBuilder;
use std::sync::{Arc,Mutex};
use tokio_curl::Session;
use std::sync::mpsc::{sync_channel,SyncSender,Receiver};
use tokio_core::reactor::Handle;
use futures::Future;
use std::sync::mpsc::{RecvError,SendError};
use super::Task;
use super::RequestDownloader;
use super::RequestDownloaderResult;
use super::Error;
use super::Config;

struct Worker {
	remote: Remote,
	session: Arc<Mutex<Session>>,
}

impl Worker {
	pub fn new() -> Worker {
		let (tx,rx) = sync_channel::<(Arc<Mutex<Session>>,Remote)>(0);
		ThreadBuilder::new().spawn(
			move || {
				let mut lp = Core::new().expect("Unable to init downloader event-loop");
				let session = Arc::new(Mutex::new(Session::new(lp.handle())));
				let remote = lp.remote();
				tx.send((session,remote)).expect("Unable to send session and remote");
				loop {
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
		};
	}
}

/// Handle for working with network manager.
#[derive(Debug)]
pub struct NetworkManagerHandle {
	task_rx: SyncSender<Task>,
	result_tx: Receiver<RequestDownloaderResult>,
}

impl NetworkManagerHandle {
	/// Aynchronous sending task to network manager.
	pub fn send(&self,task: Task) -> Result<(), SendError<Task>> {
		return self.task_rx.send(task);
	}

	/// Returns copy of task sender.
	pub fn get_sender(&self) -> SyncSender<Task> {
		return self.task_rx.clone();
	}

	/// Receives result with locking.
	pub fn recv(&self) -> Result<RequestDownloaderResult, RecvError> {
		return self.result_tx.recv();
	}
}

/// Manager for processsing request.
pub struct NetworkManager {
	remotes: Vec<Worker>,
	result_tx: SyncSender<RequestDownloaderResult>,
}

impl NetworkManager {
	/// Creates new network manager.
	pub fn start(config: &Config) -> Result<NetworkManagerHandle,Error> {
		let mut remotes = vec![];
		let (result_tx,result_rx) = sync_channel::<RequestDownloaderResult>(config.get_limit_result_channel());
		for _ in 0..config.get_thread_count() {
			remotes.push(Worker::new());
		}
		let manager = NetworkManager {
			remotes: remotes,
			result_tx: result_tx.clone(),
		};
		let (tx,rx) = sync_channel::<Task>(config.get_limit_task_channel());
		let thread_handle = ThreadBuilder::new().spawn(
			move || {
				for worker in manager.remotes.iter().cycle() {
					let task = rx.recv();
					match task {
						Ok(task) => {
							let request_result = RequestDownloader::new(task,&*worker.session.lock().unwrap(),manager.result_tx.clone());
							match request_result {
								Ok(request) => {
									worker.remote.spawn(move |_handle:&Handle|{
										request.map(|_|{()}).map_err(|_|{()})
									});
								},
								Err(request_error) => {
									manager.result_tx.send(
										Err(Error::Curl(request_error))
									).expect("Unable to send result");
								},
							}
						},
						Err(_) => {
							return;
						},
					}
				}
			}
		);
		match thread_handle {
			Ok(_) => {},
			Err(thread_error) => {
				return Err(Error::ThreadStartError(thread_error));
			}
		}
		return Ok(NetworkManagerHandle {
			task_rx: tx,
			result_tx: result_rx,
		});
	}
}