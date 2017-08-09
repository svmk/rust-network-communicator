use curl::Error as CurlError;
use std::io::Error as IOError;
use std::fmt::{Formatter,Debug,Display,Error as FormatterError};
use std::error::Error as TraitError;
use std::convert::From;

/// Errors during starting download manager or processing request.
pub enum Error<E> {
	/// IO Error
	IOError { error: IOError },

	/// Curl error - configuring or processing request.
	/// First parameter - curl error.
	Curl { error: CurlError },

	/// User defined error while Initializing
	Initialize { error: E },
}

impl <E> Debug for Error<E> {
	#[allow(unused)]
	fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
		match self {
			&Error::IOError {ref error} => {
				write!(f, "IOError: {:?}", error)
			},
			&Error::Curl {ref error} => {
				write!(f, "Curl error: {:?}", error)
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
			&Error::IOError {..} => {
				"IOError"
			},
			&Error::Curl {..} => {
				"Curl error"
			},
			&Error::Initialize {..} => {
				"Initialize error"
			},
		}
	}

    fn cause(&self) -> Option<&TraitError> {
    	match self {
			&Error::IOError {ref error} => {
				Some(error)
			},
			&Error::Curl {ref error} => {
				Some(error)
			},
			&Error::Initialize {..} => {
				None
			},
		}
    }
}

impl <E>From<IOError> for Error<E> {
	fn from(error: IOError) -> Self {
		Error::IOError {
			error: error
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
