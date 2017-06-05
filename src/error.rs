use curl::Error as CurlError;
use std::io::Error as IOError;
quick_error! {
	/// Errors during starting download manager or processing request.
	#[derive(Debug)]
	pub enum Error {
		/// Unable to start thread.
		ThreadStartError(error: IOError) {
			cause(error)
			description("Unable to start thread")
		}

		/// Curl error - configuring or processing request.
		/// First parameter - curl error.
		Curl(error: CurlError) {
			from()
			cause(error)
			description("Curl error")
		}

		/// Error within event-loop.
		/// First parameter - description.
		/// Second parameter - debug message.
		EventLoop(description: String,debug_message: String) {
			display("Event Loop error: {}",debug_message)
			description(&description)
		}
	}
}