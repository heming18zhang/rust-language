use std::io;
use std::io::BufRead;
use std::collections::HashSet;

fn main() {
		let stdin = io::stdin();
  	let mut lines = stdin.lock().lines();
  	let count = lines.next().unwrap().unwrap().trim().parse().unwrap();
  	let nums: Vec<i32> = lines
      .take(count)
      .map(|line| line.unwrap().trim().parse().unwrap())
      .collect();
    // Remove duplicates and print
  	let mut reth = HashSet::new();
  	for num in nums {
    	if reth.contains(&num) {
      } else {
      	reth.insert(num);
       //output keeping input orders
        print!("{} ", num);
      }
    }
 
}
