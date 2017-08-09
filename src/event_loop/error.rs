use std::io::Error as IOError;
use curl::MultiError;
use curl::Error as CurlError;
use std::fmt::{Formatter,Debug,Display,Error as FormatterError};
use std::error::Error as TraitError;
pub enum Error {
	PollInit {
		error: IOError,
	},
	PollRegister {
		error: IOError,
	},
	PollReRegister {
		error: IOError,
	},
	PollPooling {
		error: IOError,
	},
	UnableAddToken,
	UnableRemoveToken,
	CurlMultiInitSocket {
		error: MultiError,
	},
	CurlMultiInitTimeout {
		error: MultiError,
	},
	CurlMultiAssign {
		error: MultiError,
	},
	CurlMultiAction {
		error: MultiError,
	},
	CurlMultiAdd {
		error: MultiError,
	},
	CurlMultiRemove {
		error: MultiError,
	},
	CurlSetToken {
		error: CurlError,
	},
	CurlMultiPeroform {
		error: MultiError,
	},
	SendResultError,
}

impl Debug for Error {
	#[allow(unused)]
	fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
		match self {
			&Error::PollInit { ref error } => {
				write!(f, "PollInit: {:?}", error)
			},
			&Error::PollRegister { ref error } => {
				write!(f, "PollRegister: {:?}", error)
			},
			&Error::PollReRegister { ref error } => {
				write!(f, "PollReRegister: {:?}", error)
			},
			&Error::PollPooling { ref error } => {
				write!(f, "PollPooling: {:?}", error)
			},
			&Error::UnableAddToken => {
				write!(f, "UnableAddToken")
			},
			&Error::UnableRemoveToken => {
				write!(f, "UnableRemoveToken")
			},
			&Error::CurlMultiInitSocket { ref error } => {
				write!(f, "CurlMultiInitSocket: {:?}", error)
			},
			&Error::CurlMultiInitTimeout { ref error } => {
				write!(f, "CurlMultiInitTimeout: {:?}", error)
			},
			&Error::CurlMultiAssign { ref error } => {
				write!(f, "CurlMultiAssign: {:?}", error)
			},
			&Error::CurlMultiAction { ref error } => {
				write!(f, "CurlMultiAction: {:?}", error)
			},
			&Error::CurlMultiAdd { ref error } => {
				write!(f, "CurlMultiAdd: {:?}", error)
			},
			&Error::CurlMultiRemove { ref error } => {
				write!(f, "CurlMultiRemove: {:?}", error)
			},
			&Error::CurlSetToken { ref error } => {
				write!(f, "CurlSetToken: {:?}", error)
			},
			&Error::CurlMultiPeroform { ref error } => {
				write!(f, "CurlMultiPeroform: {:?}", error)
			},
			&Error::SendResultError => {
				write!(f, "SendResultError")
			},
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
		write!(f,"{:?}",self)
	}
}

impl TraitError for Error {
	#[allow(unused)]
	fn description(&self) -> &str {
		match self {
			&Error::PollInit { ref error } => {
				"PollInit"
			},
			&Error::PollRegister { ref error } => {
				"PollRegister"
			},
			&Error::PollReRegister { ref error } => {
				"PollReRegister"
			},
			&Error::PollPooling { ref error } => {
				"PollPooling"
			},
			&Error::UnableAddToken => {
				"UnableAddToken"
			},
			&Error::UnableRemoveToken => {
				"UnableRemoveToken"
			},
			&Error::CurlMultiInitSocket { ref error } => {
				"CurlMultiInitSocket"
			},
			&Error::CurlMultiInitTimeout { ref error } => {
				"CurlMultiInitTimeout"
			},
			&Error::CurlMultiAssign { ref error } => {
				"CurlMultiAssign"
			},
			&Error::CurlMultiAction { ref error } => {
				"CurlMultiAction"
			},
			&Error::CurlMultiAdd { ref error } => {
				"CurlMultiAdd"
			},
			&Error::CurlMultiRemove { ref error } => {
				"CurlMultiRemove"
			},
			&Error::CurlSetToken { ref error } => {
				"CurlSetToken"
			},
			&Error::CurlMultiPeroform { ref error } => {
				"CurlMultiPeroform"
			},
			&Error::SendResultError => {
				"SendResultError"
			},
		}
	}

    fn cause(&self) -> Option<&TraitError> {
    	match self {
			&Error::PollInit { ref error } => {
				Some(error)
			},
			&Error::PollRegister { ref error } => {
				Some(error)
			},
			&Error::PollReRegister { ref error } => {
				Some(error)
			},
			&Error::PollPooling { ref error } => {
				Some(error)
			},
			&Error::UnableAddToken => {
				None
			},
			&Error::UnableRemoveToken => {
				None
			},
			&Error::CurlMultiInitSocket { ref error } => {
				Some(error)
			},
			&Error::CurlMultiInitTimeout { ref error } => {
				Some(error)
			},
			&Error::CurlMultiAssign { ref error } => {
				Some(error)
			},
			&Error::CurlMultiAction { ref error } => {
				Some(error)
			},
			&Error::CurlMultiAdd { ref error } => {
				Some(error)
			},
			&Error::CurlMultiRemove { ref error } => {
				Some(error)
			},
			&Error::CurlSetToken { ref error } => {
				Some(error)
			},
			&Error::CurlMultiPeroform { ref error } => {
				Some(error)
			},
			&Error::SendResultError => {
				None
			},
		}
    }
}