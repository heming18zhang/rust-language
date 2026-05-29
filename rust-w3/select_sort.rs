use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    // 读取元素个数 N
    let n: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();
    // 读取 N 行数字放入 Vector
    let mut numbers: Vec<i32> = (0..n)
        .map(|_| lines.next().unwrap().unwrap().trim().parse().unwrap())
        .collect();

    // 正确的选择排序
    for p in 0..n-1 {
        // 1. 假设当前未排序部分的第一个元素（位置 p）就是最小值
        let mut min_idx = p;

        // 2. 从 p+1 开始，在剩余元素中寻找真正的最小值
        for m in p + 1..n {
            if numbers[m] < numbers[min_idx] {
                min_idx = m; // 记录更小元素的索引
            }
        }

        // 3. 将找到的最小值与当前位置 p 进行交换（如果 min_idx 没变，自己跟自己换也没影响）
        numbers.swap(p, min_idx);

        // 4. 打印当前步骤的数组状态（使用 &numbers 避免所有权转移）
        for num in &numbers {
            print!("{} ", num);
        }
        println!();
    }
}
