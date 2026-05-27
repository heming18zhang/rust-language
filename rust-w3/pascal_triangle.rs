use std::io;
fn pascal(n: usize) -> Vec<u32> {
	if n <= 1 {
  	return vec![1];
  } else if n == 2 {
  	return vec![1, 1];
  } else {
  	let mut retv: Vec<u32> = vec![1];
    let prev = pascal(n - 1);
    for pair in prev.windows(2) {
    	retv.push(pair[0] + pair[1]);
    }
    retv.push(1);
    return retv;
  }
}
fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let n: usize = input.trim().parse().unwrap();
		
    // Calculate and print row N of Pascal's triangle
		for num in pascal(n) {
    	print!("{} ", num);
    }
}
