use std::io;

fn main() {
	let mut input = String::new();
  io::stdin().read_line(&mut input).unwrap();
  let sa = input.trim().to_string();
  input.clear();
  io::stdin().read_line(&mut input).unwrap();
  let sb = input.trim().to_string();
    // Check if anagrams and print
  let mut letter = vec![0;26];
  for l in sa.chars() {
  	letter[l as usize - b'a' as usize] += 1;
  }
  for l in sb.chars() {
  	letter[l as usize - b'a' as usize] -= 1;
  }
  let mut ana: bool = true;
  for l in letter {
    if l > 0 {
    	ana = false;
      break;
    }
  }
  println!("{}", if ana {"Yes"} else {"No"});
}
