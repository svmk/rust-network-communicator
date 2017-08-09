use event_loop::EventLoopConfig;
/// Structure for configuration.
#[derive(Debug,Clone)]
pub struct Config {
	thread_count: usize,
	events_capacity: usize,
	limit_result_channel_buffer: usize,
	limit_task_channel_buffer: usize,
}

quick_error! {
	/// Errors in configuration.
	#[derive(Debug)]
	pub enum ConfigError {
		/// Threads count must be greater than zero.
		InvalidThreadCount {
			description("Field thread_count shall be positive")
		}
		/// Events buffer length in event-loop. Must be greater than zero.
		InvalidEventsCapacity {
			description("Field events_capacity shall be positive")
		}
	}
}


impl Config {
	/// Creates new configuration structure. 
	/// First parameter - number of threads. You can get this value from `num_cpus` crate.
	pub fn new(thread_count: usize) -> Result<Config,ConfigError> {
		let result = Config {
			thread_count: thread_count,
			events_capacity: 128,
			limit_result_channel_buffer: 0,
			limit_task_channel_buffer: 0,
		};
		result.check_configuration()?;
		Ok(result)
	}

	fn check_configuration(&self) -> Result<(),ConfigError> {
		if self.thread_count <= 0 {
			return Err(ConfigError::InvalidThreadCount);
		}
		if self.events_capacity == 0 {
			return Err(ConfigError::InvalidEventsCapacity);
		}
		Ok(())
	}

	/// Returns number of used threads.
	pub fn get_thread_count(&self) -> usize {
		return self.thread_count;
	}
	
	/// Set limit for sync channel of result values.
	pub fn set_limit_result_channel(&mut self,value: usize) -> Result<&mut Self,ConfigError> {
		self.limit_result_channel_buffer = value;
		self.check_configuration()?;
		Ok(self)
	}

	/// Returns limit for sync channel of result values.
	pub fn get_limit_result_channel(&self) -> usize {
		return self.limit_result_channel_buffer;
	}

	/// Set limit for sync channel of tasks.
	pub fn set_limit_task_channel(&mut self,value: usize) -> Result<&mut Self,ConfigError> {
		self.limit_task_channel_buffer = value;
		self.check_configuration()?;
		Ok(self)
	}

	/// Returns limit for sync channel of tasks.
	pub fn get_limit_task_channel(&self) -> usize {
		return self.limit_task_channel_buffer;
	}

	/// Set events buffer length.
	pub fn set_events_capacity(&mut self, value: usize) -> Result<&mut Self,ConfigError> {
		self.events_capacity = value;
		self.check_configuration()?;
		Ok(self)
	}

	/// Returns events capacity.
	pub fn get_events_capacity(&self) -> usize {
		return self.events_capacity;
	}
}

pub fn get_event_loop_config(config: &Config) -> EventLoopConfig {
	return EventLoopConfig::new(config.events_capacity);
}