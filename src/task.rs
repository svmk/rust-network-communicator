use curl::easy::Easy2;
use std::boxed::Box;
use super::Error;

enum InnerTask<T,E> {
	Task {
		configurator: Box<Fn() -> Result<Easy2<T>,Error<E>> + Send + 'static>
	},
	Terminate
}

/// Task for network manager.
pub struct Task<T,E> {
	inner: InnerTask<T,E>,
}

impl <T,E>Task<T,E> {
	/// Creates new task with payload
	pub fn new<F>(configurator: F) -> Task<T,E> where F: Send + 'static, F: Fn() -> Result<Easy2<T>,Error<E>> {
		let task = Task {
			inner: InnerTask::Task {
				configurator: Box::new(configurator),
			}
		};
		return task;
	}

	pub fn build(self) -> Result<Easy2<T>,Error<E>> {
		match self.inner {
			InnerTask::Task {ref configurator} => {
				return (configurator)();
			},
			InnerTask::Terminate => {
				// Never happens.
				panic!("Build called in terminate state");
			}
		}
	}
}

pub fn is_terminate_task<T,E>(task: &Task<T,E>) -> bool {
	match task.inner {
		InnerTask::Terminate => true,
		InnerTask::Task {..} => false,
	}
}

pub fn termination_task<T,E>() -> Task<T,E> {
	Task {
		inner: InnerTask::Terminate,
	}
}