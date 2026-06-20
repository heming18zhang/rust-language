#![allow(clippy::ptr_arg)]

// TODO: Fix the compiler errors without changing anything except adding or
// removing references (the character `&`).

// Shouldn't take ownership
fn get_char(data: &String) -> char {
    //let data = data.clone();
    data.chars().last().unwrap()
}

// Should take ownership
fn string_uppercase(data: &mut String) {
     let ud = data.to_uppercase();

    println!("{data}");
}

fn main() {
    let mut data = "Rust is great!".to_string();

    get_char(&data);

    string_uppercase(&mut data);
}
