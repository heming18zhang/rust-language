use std::io;

fn main() {
	let mut input = String::new();
  io::stdin().read_line(&mut input).unwrap();
  let nums: String = input.trim().to_string();
  input.clear();
  io::stdin().read_line(&mut input).unwrap();
  let sbase: i32 = input.trim().parse().unwrap();
  input.clear();
  io::stdin().read_line(&mut input).unwrap();
  let tbase: i32 = input.trim().parse().unwrap();
    // Convert and print
  let mut temp:i128 = 0;
  for n in nums.chars() {
  	temp = temp * sbase as i128 + n.to_digit(10).unwrap() as i128;
  }
  let mut tstr = String::new();
  while temp > 0 {
  	let digi: i32 = temp as i32 % tbase;
    tstr.push_str(&digi.to_string());
    temp /= tbase as i128;
  }
  println!("{}", tstr.chars().rev().collect::<String>());
}
