use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let n: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    let mut matrix = vec![vec![0i64; n]; n];
    for i in 0..n {
        let line = lines.next().unwrap().unwrap();
        let vals: Vec<i64> = line.trim().split(' ').map(|x| x.parse().unwrap()).collect();
        for j in 0..n { matrix[i][j] = vals[j]; }
    }
		let mut sum = 0;
    // Calculate and print the diagonal sum
  	for i in 0..n {
    	for j in 0..n {
      	if i == j || i + j + 1 == n {
        	sum += matrix[i][j];
        }
      }
    }
  	println!("{}", sum);
}
