use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let n: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    let mut numbers: Vec<i32> = (0..n).map(|_| lines.next().unwrap().unwrap().trim().parse().unwrap()).collect();

    // Bubble sort with swap counting
  	
 	let len = numbers.len() - 1;
  	let mut swaps = 0;
  	for i in 0..len {
    	for j in 0..len-i-1 {
      	if numbers[j] > numbers[j+1] {
        	numbers.swap(j, j+1);
          swaps += 1;
        } 
      }
    }
  	for n in numbers {
    	print!("{} ", n);
    }
  	println!("");
  	println!("Swaps: {}", swaps);
}
