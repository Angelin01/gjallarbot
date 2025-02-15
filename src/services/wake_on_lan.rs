use crate::errors::InvalidMacError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::Ipv4Addr;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use tokio::io;
use tokio::net::UdpSocket;

const MAC_ADDRESS_SIZE: usize = 6;
const HEADER_SIZE: usize = 6;
const MAC_REPETITIONS: usize = 16;
const MAGIC_PACKET_SIZE: usize = HEADER_SIZE + (MAC_ADDRESS_SIZE * MAC_REPETITIONS);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct MacAddress(pub [u8; MAC_ADDRESS_SIZE]);

impl FromStr for MacAddress {
	type Err = InvalidMacError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts: Vec<_> = s.split(":").collect();
		let len = parts.len();
		if len != MAC_ADDRESS_SIZE {
			return Err(InvalidMacError::WrongPartCount {
				expected: MAC_ADDRESS_SIZE,
				actual: len,
			});
		}

		let mut mac = [0; MAC_ADDRESS_SIZE];

		for (i, part) in parts.iter().enumerate() {
			mac[i] = u8::from_str_radix(part, 16)
				.map_err(|_| InvalidMacError::InvalidHexString(part.to_string()))?;
		}

		Ok(Self(mac))
	}
}

impl fmt::Display for MacAddress {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let parts: Vec<String> = self.0.iter().map(|byte| format!("{:02X}", byte)).collect();
		write!(f, "{}", parts.join(":"))
	}
}

impl Deref for MacAddress {
	type Target = [u8; MAC_ADDRESS_SIZE];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for MacAddress {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MagicPacket([u8; MAGIC_PACKET_SIZE]);

impl MagicPacket {
	pub fn from_mac(mac: &MacAddress) -> Self {
		let mut magic_packet = [0xFFu8; MAGIC_PACKET_SIZE];

		for repetition in 0..MAC_REPETITIONS {
			let offset_start = HEADER_SIZE + repetition * MAC_ADDRESS_SIZE;
			let offset_end = offset_start + MAC_ADDRESS_SIZE;
			magic_packet[offset_start..offset_end].copy_from_slice(mac.deref());
		}

		Self(magic_packet)
	}

	pub async fn send(&self) -> io::Result<()> {
		let src = (Ipv4Addr::new(0, 0, 0, 0), 0);
		let dst = (Ipv4Addr::new(255, 255, 255, 255), 9);
		let socket = UdpSocket::bind(src).await?;
		socket.set_broadcast(true)?;
		socket.send_to(self.deref(), dst).await?;

		Ok(())
	}
}

impl Deref for MagicPacket {
	type Target = [u8; MAGIC_PACKET_SIZE];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for MagicPacket {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub trait MagicPacketSender {
	async fn send(&self, magic_packet: &MagicPacket) -> io::Result<()>;
}

pub struct UdpMagicPacketSender;
impl MagicPacketSender for UdpMagicPacketSender {
	async fn send(&self, magic_packet: &MagicPacket) -> io::Result<()> {
		let src = (Ipv4Addr::new(0, 0, 0, 0), 0);
		let dst = (Ipv4Addr::new(255, 255, 255, 255), 9);
		let socket = UdpSocket::bind(src).await?;
		socket.set_broadcast(true)?;
		socket.send_to(&**magic_packet, dst).await?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[tokio::test]
	async fn given_invalid_mac_with_non_hex_characters_then_returns_invalid_hex_string_error() {
		let result = MacAddress::from_str("AA:BB:CC:DD:EE:GG");
		assert!(result.is_err());

		let error = result.unwrap_err();

		assert!(matches!(error, InvalidMacError::InvalidHexString(_)));
	}

	#[tokio::test]
	async fn given_mac_with_incorrect_number_of_parts_then_returns_wrong_part_count_error() {
		let result = MacAddress::from_str("AA:BB:CC:DD:EE");
		assert!(result.is_err());

		let error = result.unwrap_err();
		assert_eq!(
			error,
			InvalidMacError::WrongPartCount {
				expected: MAC_ADDRESS_SIZE,
				actual: 5
			}
		);
	}

	#[tokio::test]
	async fn given_a_mac_address_then_it_is_formatted_properly() {
		let mac_address = MacAddress([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
		assert_eq!(format!("{}", mac_address), "AA:BB:CC:DD:EE:FF");
	}

	#[tokio::test]
	async fn given_valid_mac_address_then_builds_correct_magic_packet() {
		let mac_address = MacAddress([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
		let magic_packet = MagicPacket::from_mac(&mac_address);

		#[rustfmt::skip]
		let expected_magic_packet = [
			0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
			0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
		];

		assert_eq!(*magic_packet, expected_magic_packet);
	}
}
