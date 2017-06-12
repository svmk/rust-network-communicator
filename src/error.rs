use curl::Error as CurlError;
use std::io::Error as IOError;
use std::fmt::{Formatter,Debug,Display,Error as FormatterError};
use std::error::Error as TraitError;
use std::convert::From;

/// Errors during starting download manager or processing request.
pub enum Error<E> {
	/// Unable to start thread.
	ThreadStartError { error: IOError },

	/// Curl error - configuring or processing request.
	/// First parameter - curl error.
	Curl { error: CurlError },

	/// Error within event-loop.
	/// First parameter - description.
	/// Second parameter - debug message.
	EventLoop { description: String, debug_message: String },

	/// User defined error while Initializing
	Initialize { error: E },
}

impl <E> Debug for Error<E> {
	#[allow(unused)]
	fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
		match self {
			&Error::ThreadStartError {ref error} => {
				write!(f, "Unable to start thread: {:?}", error)
			},
			&Error::Curl {ref error} => {
				write!(f, "Curl error: {:?}", error)
			},
			&Error::EventLoop {ref description,ref debug_message} => {
				write!(f, "Event loop error: {}", debug_message)
			},
			&Error::Initialize {..} => {
				write!(f, "Initialize error")
			},
		}
	}
}

impl <E> Display for Error<E> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
		write!(f,"{:?}",self)
	}
}

impl <E>TraitError for Error<E> {
	#[allow(unused)]
	fn description(&self) -> &str {
		match self {
			&Error::ThreadStartError {..} => {
				"Unable to start thread"
			},
			&Error::Curl {..} => {
				"Curl error"
			},
			&Error::EventLoop {ref description,ref debug_message} => {
				&description
			},
			&Error::Initialize {..} => {
				"Initialize error"
			},
		}
	}

    fn cause(&self) -> Option<&TraitError> {
    	match self {
			&Error::ThreadStartError {ref error} => {
				Some(error)
			},
			&Error::Curl {ref error} => {
				Some(error)
			},
			&Error::EventLoop {..} => {
				None
			},
			&Error::Initialize {..} => {
				None
			},
		}
    }
}


impl <E>From<CurlError> for Error<E> {
	fn from(error: CurlError) -> Self {
		Error::Curl {
			error: error
		}
	}
}


// impl <E: !CurlError>From<E> for Error<E> where E: !CurlError {
// 	fn from(error: E) -> Self {
// 		Error::Initialize {
// 			error: error
// 		}
// 	}
// }
