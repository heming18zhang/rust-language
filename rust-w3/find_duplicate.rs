use std::io;
use std::collections::HashSet;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let n: usize = input.trim().parse().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let arr: Vec<i32> = input.trim().split(' ').map(|x| x.parse().unwrap()).collect();

    // Find and print the duplicate number
  	let mut dup: Vec<i32> = vec![0;n];
  	for a in arr {
      dup[a as usize - 1] += 1;
    }
  	for (i, &v) in dup.iter().enumerate() {
    	if v > 1 {
        println!("{}", i + 1);
      }
    }
}
