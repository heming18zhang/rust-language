use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let n1: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    let list1: Vec<i32> = (0..n1).map(|_| lines.next().unwrap().unwrap().trim().parse().unwrap()).collect();

    let n2: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    let list2: Vec<i32> = (0..n2).map(|_| lines.next().unwrap().unwrap().trim().parse().unwrap()).collect();

    // Merge and print
  	let mut idx1: usize = 0;
  	let mut idx2: usize = 0;
  	let mut retv: Vec<i32> = vec![];
  	while idx1 < n1 && idx2 < n2 {
    	if list1[idx1] < list2[idx2] {
      	retv.push(list1[idx1]);
        idx1 += 1;
      } else {
      	retv.push(list2[idx2]);
        idx2 += 1;
      }
    }
  	while idx1 < n1 {
    	retv.push(list1[idx1]);
      idx1 += 1;
    }
  	while idx2 < n2 {
    	retv.push(list2[idx2]);
      idx2 += 1;
    }
  	for num in retv {
    	print!("{} ", num);
    }
}
