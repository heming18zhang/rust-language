use std::io;
fn isprime(n: u32) -> bool {
	if n <= 1 {
  	return false;
  }
  if n == 2 {
  	return true;
  }
  for i in 2..=n/2 {
  	if n % i == 0 {
    	return false;
    }
  }
  true
}
fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let n: usize = input.trim().parse().unwrap();

    // Use the Sieve of Eratosthenes to find and sum all primes up to n
  	let mut sum = 0;
  	for i in 1..=n {
    	if isprime(i as u32) {
      	sum += i;
      }
    }
  println!("{}", sum);
}
