use std::io;

// Determines if a year is a leap year
fn leap(y: i32) -> bool {
    y % 400 == 0 || (y % 4 == 0 && y % 100 != 0)
}

fn main() {
    // 1. Read Day
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let day: i32 = input.trim().parse().unwrap();

    // 2. Read Month
    input.clear(); // Reusing the buffer cleanly!
    io::stdin().read_line(&mut input).unwrap();
    let month: i32 = input.trim().parse().unwrap();

    // 3. Read Year
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let year: i32 = input.trim().parse().unwrap();

    // 4. Validate Date
    let days = vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let ldays = vec![31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    
    let mut valid = true;

    if month < 1 || month > 12 {
        valid = false;
    } else {
        let max_days = if leap(year) {
            ldays[month as usize - 1]
        } else {
            days[month as usize - 1]
        };

        if day < 1 || day > max_days {
            valid = false;
        }
    }

    println!("{}", if valid { "Valid" } else { "Invalid" });
}
