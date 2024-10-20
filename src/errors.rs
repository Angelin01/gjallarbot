use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum InvalidMacError {
	#[error("Expected {expected} parts in MAC address, but got {actual}")]
	WrongPartCount { expected: usize, actual: usize },

	#[error("Invalid hexadecimal string")]
	InvalidHexString(#[from] ParseIntError),
}
