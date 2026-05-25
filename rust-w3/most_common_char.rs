use std::io;
use std::collections::HashMap;

fn main() {

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let text = input.trim().to_lowercase();
    
    let mut char_count = HashMap::new();
    for ch in text.chars() {
        *char_count.entry(ch).or_insert(0) += 1;
    }

    // 使用 .max_by() 编写确定性对比逻辑
    let (&letter, _) = char_count
        .iter()
        .max_by(|&(char_a, count_a), &(char_b, count_b)| {
            // 1. 首先对比出现次数（次数大的优先）
            match count_a.cmp(count_b) {
                std::cmp::Ordering::Equal => {
                    // 2. 如果次数并列，对比字符的字典序！
                    // 注意：因为我们要找的是“字典序小”的，而 max_by 找的是“最大值”
                    // 所以我们要反向对比，让小字符（如 'a'）在对比中胜出
                    char_b.cmp(char_a) 
                }
                other => other,
            }
        })
        .unwrap();

    println!("{}", letter);
}
