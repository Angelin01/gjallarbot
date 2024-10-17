#![feature(trait_alias)]

use serde::{Deserialize, Serialize};

mod persistent_data;
mod wake_on_lan;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
	setting1: String,
	setting2: u32,
}

fn main() {
	let mac =  "D8:43:AE:57:B4:1D";
	let magic_packet = wake_on_lan::MagicPacket::from_string(mac).unwrap();
	wake_on_lan::send(&magic_packet).unwrap();

	// let path = PathBuf::from("config.json");
	// let mut persistent_data = PersistentJson::<Config>::new(path).expect("Failed to load data");
	// {
	// 	let read_data = &*persistent_data;
	// 	println!("Setting 1: {:?}", read_data.setting1);
	// 	println!("Setting 2: {:?}", read_data.setting2);
	// }
	//
	// {
	// 	let mut write_data = persistent_data.write();
	// 	write_data.setting1 = "New Value".to_string();
	// 	write_data.setting2 += 1;
	// }
}
