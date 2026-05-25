use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut line = stdin.lock().lines();
    
    // 1. 连续两个 unwrap 稳稳解包第一行
    let cnt: usize = line.next().unwrap().unwrap().trim().parse().unwrap();
    
    // 2. 收集 N 行字符串，并【掐头去尾】清理换行符
    let mut word: Vec<String> = line.take(cnt)
        .map(|s| s.unwrap().trim().to_string()) // 🟢 关键：干净利落，去除 \r\n
        .collect(); 
        
    // 3. 性能压榨就地排序（对 String 完全可用！）
    word.sort_unstable();
    
    // 4. 干净无括号逐行打印
    word.iter().for_each(|w| println!("{}", w));
}