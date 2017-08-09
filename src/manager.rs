use std::thread::Builder as ThreadBuilder;
use std::thread::JoinHandle;
use std::mem::swap as swap_variable;
use std::sync::{Arc};
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{sync_channel,SyncSender,Receiver};
use std::sync::mpsc::{RecvError,SendError};
use std::sync::atomic::Ordering as AtomicOrdering;
use std::ops::Drop;
#[allow(deprecated)]
use mio::channel::channel;
use event_loop::EventLoop;
use curl::easy::Easy2;
use config::get_event_loop_config;
use task::{termination_task,is_terminate_task};
use super::Task;
use super::Error;
use super::Config;


/// Handle for working with network manager.
pub struct NetworkManagerHandle<T,E> {
	managers: Vec<NetworkManager>,
	result_rx: Receiver<Result<Easy2<T>,Error<E>>>,
	task_tx: SyncSender<Task<T,E>>,
	collector: Option<JoinHandle<()>>,
}

impl <T,E>NetworkManagerHandle<T,E> {
	/// Aynchronous sending task to network manager.
	pub fn send(&self,task: Task<T,E>) -> Result<(), SendError<Task<T,E>>> {
		return self.task_tx.send(task);
	}

	/// Returns copy of task sender.
	pub fn get_sender(&self) -> SyncSender<Task<T,E>> {
		return self.task_tx.clone();
	}

	/// Receives result with locking.
	pub fn recv(&self) -> Result<Result<Easy2<T>,Error<E>>, RecvError> {
		return self.result_rx.recv();
	}

	/// Returns count of active requests.
	pub fn get_active_requests(&self) -> usize {
		let mut result = 0usize;
		for manager in self.managers.iter() {
			result = result + manager.active_requests.load(
				AtomicOrdering::Relaxed
			);
		}
		return result;
	}
}


impl <T,E>Drop for NetworkManagerHandle<T,E> {
    fn drop(&mut self) {
    	self.task_tx.send(
    		termination_task()
		).expect(
			"Unable to send termination task"
		);
    	let mut collector = None;
    	swap_variable(&mut self.collector, &mut collector);
    	collector.unwrap().join().expect(
			"Unable to join thread"
		);
    	for manager in self.managers.drain(..) {
    		manager.thread_handle.join(
			).expect(
				"Unable to join thread"
			);
    	}
    }
}

/// Manager for processsing request.
pub struct NetworkManager {
	active_requests: Arc<AtomicUsize>,
	thread_handle: JoinHandle<()>,
}

impl NetworkManager {
	/// Creates new network manager.
	/// Produces threads that may panic when something is going wrong.
	pub fn start<T,E>(config: &Config) -> Result<NetworkManagerHandle<T,E>,Error<E>>  where E: Send + 'static, T: Send + 'static {
		let (result_tx, result_rx) = sync_channel::<Result<Easy2<T>,Error<E>>>(config.get_limit_result_channel());
		let mut managers = Vec::with_capacity(config.get_thread_count());
		let mut task_tx_items = Vec::with_capacity(config.get_thread_count());
		for _ in 0..config.get_thread_count() {
			let active_requests = Arc::new(AtomicUsize::new(0));
			let active_requests_thread = active_requests.clone();
			#[allow(deprecated)]
			let (task_tx, task_rx) = channel();
			task_tx_items.push(task_tx.clone());
			let thread_result_tx = result_tx.clone();
			let worker_config = get_event_loop_config(config);
			let handle = ThreadBuilder::new().spawn(move || {
				let mut worker = EventLoop::new(
					worker_config, task_rx, thread_result_tx
				).expect("Unable to initialize worker");
				worker.set_executing_requests_handler(move |active_requests|{
					active_requests_thread.store(
						active_requests as usize,
						AtomicOrdering::Relaxed
					);
				});
				worker.start().expect("Error in event loop");
			}).map_err(|e|{
				Error::IOError {error: e}
			})?;
			managers.push(NetworkManager {
				active_requests: active_requests,
				thread_handle: handle,
			});
		}
		let (task_tx, task_rx) = sync_channel(config.get_limit_task_channel());
		let collector = ThreadBuilder::new().spawn(move || {
			for task_channel in task_tx_items.iter().cycle() {
				let task = task_rx.recv().expect(
					"Unable to receive data"
				);
				if is_terminate_task(&task) {
					break;
				}
				#[allow(deprecated)]
				task_channel.send(
					task
				).expect(
					"Unable to send task"
				);
			}
			for task_channel in task_tx_items.iter() {
				#[allow(deprecated)]
				task_channel.send(
					termination_task()
				).expect(
					"Unable to send task"
				);
			}
		}).map_err(|e|{
			Error::IOError {error: e}
		})?;
		Ok(NetworkManagerHandle{
			managers: managers,
			result_rx: result_rx,
			task_tx: task_tx,
			collector: Some(collector),
		})
	}
}