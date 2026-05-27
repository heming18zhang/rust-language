use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let n: usize = input.trim().parse().unwrap();

    // 1. Print the top half of the diamond (including the center row)
    for j in (1..=n).step_by(2) {
        for _ in 1..=(n - j) / 2 {
            print!(" ");
        }
        for _ in 1..=j {
            print!("*");
        }
        println!();
    }

    // 2. Print the bottom half of the diamond
    for j in (1..n).step_by(2).rev() {
        for _ in 1..=(n - j) / 2 {
            print!(" ");
        }
        for _ in 1..=j {
            print!("*");
        }
        println!();
    }
}
