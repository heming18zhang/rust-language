use std::io;

fn main() {
		let mut input = String::new();
  	io::stdin().read_line(&mut input).unwrap();
  	let s = input.trim().to_string();
    // Compress and print
  	let mut prev: Option<char> = None;
  	let mut rpt = 0;
  	let mut rets = String::new();
  	for ch in s.chars() {
    	if prev == None {
        rets.push(ch);
        rpt = 1;
        prev = Some(ch);
      } else if Some(ch) == prev {
        rpt += 1;  
      } else {
      	rets.push_str(&rpt.to_string());
        rets.push(ch);
        rpt = 1;
        prev = Some(ch);
      }
    }
  	rets.push_str(&rpt.to_string());
  	println!("{}", rets);
}
