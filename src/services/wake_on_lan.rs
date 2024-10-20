use std::net::{Ipv4Addr, UdpSocket};
use anyhow::Result;

const MAC_ADDRESS_SIZE: usize = 6;
pub type MacAddress = [u8; MAC_ADDRESS_SIZE];
const HEADER_SIZE: usize = 6;
const MAC_REPETITIONS: usize = 16;
const MAGIC_PACKET_SIZE: usize = HEADER_SIZE + (MAC_ADDRESS_SIZE * MAC_REPETITIONS);

// TODO: rewrite this into an anonymous type with defer
pub struct MagicPacket {
	raw_bytes: [u8; MAGIC_PACKET_SIZE],
}

impl MagicPacket {
	pub fn from_string<T: AsRef<str>>(mac: T) -> Result<Self> {
		let bytes_mac = Self::parse_mac(mac)?;
		let magic_packet = Self::build_magic_packet(&bytes_mac);
		Ok(Self { raw_bytes: magic_packet })
	}

	pub fn bytes(&self) -> &[u8; MAGIC_PACKET_SIZE] {
		&self.raw_bytes
	}

	fn parse_mac<T: AsRef<str>>(mac: T) -> Result<MacAddress> {
		let parts: Vec<_> = mac.as_ref().split(":").collect();
		let len = parts.len();
		if len != MAC_ADDRESS_SIZE {
			return Err(anyhow::Error::msg(format!("Invalid MAC address: {} parts", len)));
		}

		let mut mac: MacAddress = [0; MAC_ADDRESS_SIZE];

		for (i, part) in parts.iter().enumerate() {
			mac[i] = u8::from_str_radix(part, 16)
				.map_err(|_| anyhow::Error::msg(format!("Invalid hex value in MAC address: {}", part)))?;
		}

		Ok(mac)
	}

	fn build_magic_packet(mac: &MacAddress) -> [u8; 102] {
		let mut magic_packet = [0xFFu8; MAGIC_PACKET_SIZE];

		for repetition in 0..MAC_REPETITIONS {
			let offset_start = HEADER_SIZE + repetition * MAC_ADDRESS_SIZE;
			let offset_end = offset_start + MAC_ADDRESS_SIZE;
			magic_packet[offset_start..offset_end].copy_from_slice(mac);
		}

		magic_packet
	}
}

pub fn send(magic_packet: &MagicPacket) -> Result<()> {
	let src = (Ipv4Addr::new(0, 0, 0, 0), 0);
	let dst = (Ipv4Addr::new(255, 255, 255, 255), 9);
	let socket = UdpSocket::bind(src)?;
	socket.set_broadcast(true)?;
	socket.send_to(magic_packet.bytes(), dst)?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn given_invalid_mac_with_non_hex_characters_then_returns_error() {
		let result = MagicPacket::from_string("AA:BB:CC:DD:EE:GG"); // 'GG' is not valid hex
		assert!(result.is_err());
	}

	#[test]
	fn given_mac_with_incorrect_number_of_parts_then_returns_error() {
		let result = MagicPacket::from_string("AA:BB:CC:DD:EE"); // Only 5 parts instead of 6
		assert!(result.is_err());
	}

	#[test]
	fn given_valid_mac_address_then_builds_correct_magic_packet() {
		let result = MagicPacket::from_string("AA:BB:CC:DD:EE:FF").unwrap();
		let magic_packet = result.bytes();

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

		assert_eq!(magic_packet, &expected_magic_packet);
	}
}
