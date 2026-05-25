use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let mut letters: Vec<i32> = vec![0; 26];
    
    for ch in input.trim().chars() {
        // 🛠️ 修复 1：在计算索引前，必须确保字符在 'a'..='z' 范围内
        if ch.is_ascii_lowercase() {
            letters[ch as usize - 'a' as usize] += 1;
        }
        // 如果想兼容大写字母，也可以加上这一句：
        // else if ch.is_ascii_uppercase() {
        //     letters[ch as usize - 'A' as usize] += 1;
        // }
    }
    
    for (i, l) in letters.iter().enumerate() {
        if *l > 0 {
            // 🛠️ 修复 2：使用更安全干净的 char 转换方式
            let letter_char = (b'a' + i as u8) as char;
            println!("{}:{}", letter_char, l);
        }
    }
}