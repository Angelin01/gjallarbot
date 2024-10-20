use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum InvalidMacError {
	#[error("Expected {expected} parts in MAC address separated by `:`, but got {actual}")]
	WrongPartCount { expected: usize, actual: usize },

	#[error("Invalid hexadecimal value {0}")]
	InvalidHexString(String),
}
