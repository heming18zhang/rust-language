use std::io;
use std::io::BufRead;
use std::collections::HashMap;
fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let n: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    let numbers: Vec<i32> = (0..n).map(|_| lines.next().unwrap().unwrap().trim().parse().unwrap()).collect();
		let mut nset = HashMap::new();
    // Count and print unique numbers
  	for n in &numbers {
    	*nset.entry(n).or_insert(0) += 1;
    }
  	let mut uniq = 0;
  	for (_, val) in nset {
    	if val == 1 {
      	uniq += 1;
      }
    }
  	println!("Unique: {}", uniq);
}
