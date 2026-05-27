use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let n: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    let numbers: Vec<i32> = (0..n).map(|_| lines.next().unwrap().unwrap().trim().parse().unwrap()).collect();
    let target: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    // Find and print pairs
  	for i in 0..=(n-1) {
    	for j in i+1..=(n-1) {
      	if numbers[i] + numbers[j] == target {
        	println!("{} {}", numbers[i], numbers[j]);
          return;
        }
      }
    }
  	println!("No pair found");
}
