use std::io;
use std::collections::BTreeMap;

fn main() {
	let mut input = String::new();
  io::stdin().read_line(&mut input).unwrap();
  let text = input.trim().to_string();
    // Count and print word frequencies
  let mut texth = BTreeMap::new();
  for word in text.split_whitespace() {
  	*texth.entry(word).or_insert(0) += 1;
  }
  
  for (key, val) in texth {
  	println!("{}:{}", key, val);
  }
}
