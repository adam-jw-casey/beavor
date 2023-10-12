use std::process::Command;

fn main() {
    Command::new("make")
        .status()
        .expect("Failed to run make");
}
