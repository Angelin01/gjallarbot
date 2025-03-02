use std::collections::BTreeSet;
use std::fmt::Display;

pub mod wake_on_lan;
pub mod servitor;

fn format_list<T: Display, F: Fn(&T) -> String>(list: &BTreeSet<T>, formatter: F) -> String {
	if list.is_empty() {
		"None".to_string()
	} else {
		list.iter().map(formatter).collect::<Vec<_>>().join(", ")
	}
}
