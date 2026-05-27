use std::io::{self, BufRead};
use std::collections::BTreeSet;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    let n: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    
    // Use a BTreeSet to automatically sort and remove duplicates
    let mut unique_numbers = BTreeSet::new();
    
    for _ in 0..n {
        if let Some(Ok(line)) = lines.next() {
            let num: i32 = line.trim().parse().unwrap();
            unique_numbers.insert(num);
        }
    }

    // Convert to a vector so we can easily index it from the back
    let sorted_unique: Vec<&i32> = unique_numbers.iter().collect();
    let len = sorted_unique.len();

    // If we have 2 or more UNIQUE numbers, the second largest is at len - 2
    if len >= 2 {
        println!("Second largest: {}", sorted_unique[len - 2]);
    } else {
        // If all numbers were identical (like 5, 5, 5), len will be 1.
        // If the platform's fallback for "no second largest" is the max value, print that.
        println!("Second largest: {}", sorted_unique[0]);
    }
}
