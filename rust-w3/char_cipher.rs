use std::io;

fn main() {
    // 1. 读取待加密的纯小写文本
    let mut text = String::new();
    io::stdin().read_line(&mut text).unwrap();
    let text = text.trim();

    // 2. 读取位移量（提前对 26 取模，彻底免疫后续 u8 溢出风险）
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let shift = (input.trim().parse::<u32>().unwrap() % 26) as u8;

    // 3. 执行流式加密并收集结果
    let rets: String = text
        .chars()
        .map(|c| ((c as u8 - b'a' + shift) % 26 + b'a') as char)
        .collect();

    // 4. 打印最终密文
    println!("{}", rets);
}