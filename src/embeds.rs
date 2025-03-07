use poise::serenity_prelude::{Colour, CreateEmbed};

pub fn success<S: AsRef<str>>(title: S, description: impl Into<String>) -> CreateEmbed {
	pub const COLOR_SUCCESS: Colour = Colour(0x77b255);
	pub const EMOJI_SUCCESS: &str = ":white_check_mark:";
	let title = title.as_ref();
	create_embed(format!("{EMOJI_SUCCESS} {title}"), description, COLOR_SUCCESS)
}

pub fn error<S: AsRef<str>>(title: S, description: impl Into<String>) -> CreateEmbed {
	pub const COLOR_ERROR: Colour = Colour(0xdd2e44);
	pub const EMOJI_ERROR: &str = ":x:";
	let title = title.as_ref();
	create_embed(format!("{EMOJI_ERROR} {title}"), description, COLOR_ERROR)
}

pub fn info<S: AsRef<str>>(title: S, description: impl Into<String>) -> CreateEmbed {
	pub const COLOR_INFO: Colour = Colour(0x55acee);
	pub const EMOJI_INFO: &str = ":information_source:";
	let title = title.as_ref();
	create_embed(format!("{EMOJI_INFO} {title}"), description, COLOR_INFO)
}

pub fn internal_error<S: AsRef<str>>(title: S, description: impl Into<String>) -> CreateEmbed {
	pub const COLOR_INTERNAL: Colour = Colour(0xF4900C);
	pub const EMOJI_INTERNAL: &str = ":tools:";
	let title = title.as_ref();
	create_embed(format!("{EMOJI_INTERNAL} {title}"), description, COLOR_INTERNAL)
}

pub fn invalid_machine<S: AsRef<str>>(machine_name: S) -> CreateEmbed {
	let name = machine_name.as_ref();
	error("Invalid Machine", format!("No machine with name {name} exists"))
}

pub fn invalid_servitor_server<S: AsRef<str>>(server_name: S) -> CreateEmbed {
	let name = server_name.as_ref();
	error(
		"Invalid Servitor server",
		format!("No Servitor server with name {name} exists"),
	)
}

fn create_embed(title: impl Into<String>, description: impl Into<String>, color: impl Into<Colour>) -> CreateEmbed {
	CreateEmbed::default()
		.title(title)
		.description(description)
		.colour(color)
}
