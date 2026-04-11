use std::ops::{Deref, DerefMut};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum InvalidMacError {
	#[error("Expected {expected} parts in MAC address separated by `:`, but got {actual}")]
	WrongPartCount { expected: usize, actual: usize },

	#[error("Invalid hexadecimal value {0}")]
	InvalidHexString(String),
}

#[derive(Debug)]
#[repr(transparent)]
pub struct UnexpectedError(pub anyhow::Error);

impl PartialEq for UnexpectedError {
	fn eq(&self, _other: &Self) -> bool {
		false
	}
}

impl Deref for UnexpectedError {
	type Target = anyhow::Error;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for UnexpectedError {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}