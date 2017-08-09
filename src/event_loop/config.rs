/// Structure for event-loop configuration.
#[derive(Debug,Clone)]
pub struct Config {
	events_capacity: usize,
}

impl Config {
	/// Creates new event loop config
	pub fn new(events_capacity: usize) -> Config {
		Config {
			events_capacity: events_capacity,
		}
	}

	/// Returns events capacity.
	pub fn get_events_capacity(&self) -> usize {
		return self.events_capacity;
	}
}