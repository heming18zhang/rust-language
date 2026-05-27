use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let first_name = input.trim().to_string();

    input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let last_name = input.trim().to_string();

    // Create username (lowercase, no space)
		let username = format!("{}{}", &first_name, &last_name).to_lowercase();
    // Create initials (uppercase first letters)
		let initname = format!("{}{}", &first_name[0..1], &last_name[0..1]).to_uppercase();
    // Print results
  	println!("Username: {}\nInitials: {}", username, initname);
}
