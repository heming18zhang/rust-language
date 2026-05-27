use std::io;

fn gcd(a: i32, b: i32) -> i32 {
    // Implement the Euclidean algorithm recursively
  	let mut divider = if a < b {a} else {b};
  	
  	while divider >= 2 {
    	if a % divider == 0 && b % divider == 0 {
      	break;
      } 
			divider -= 1;
    }
    divider
}

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let a: i32 = input.trim().parse().unwrap();

    input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let b: i32 = input.trim().parse().unwrap();

    println!("GCD: {}", gcd(a, b));
}
