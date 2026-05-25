use std::io;

fn main() {
    let mut num = String::new();
    io::stdin().read_line(&mut num).unwrap();
    let num = num.trim();
    
    // 必须加上 mut，否则后续无法修改数组内容
    let mut digits: Vec<u8> = vec![0; 10]; 
    
    for chd in num.chars() {
        // 必须安全处理 to_digit 的 Option 返回值，并防范非数字噪声
        if let Some(digit_idx) = chd.to_digit(10) {
            digits[digit_idx as usize] += 1;
        }
    }
    
    for (i, &d) in digits.iter().enumerate() {
        if d > 0 {
            println!("{}:{}", i, d);
        }
    }
}