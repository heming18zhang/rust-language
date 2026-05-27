use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    // 1. 读取总行数
    let count: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();

    // 2. 读取数组（用 .by_ref() 留住 lines 后面继续用）
    let nums: Vec<i32> = lines
        .by_ref()
        .take(count as usize)
        .map(|line| line.unwrap().trim().parse().unwrap())
        .collect();

    // 3. 读取移动步数 k
    let k: i32 = lines.next().unwrap().unwrap().trim().parse().unwrap();
    
    let len = nums.len();
    if len == 0 { return; } // 防御性代码：防止数组为空时取模报错

    // 4. 核心安全处理：提前对 k 取模。
    // 无论用户输入多大的 k，都能安全地约束在 0 到 len-1 之间
    let k_adapted = (k as usize) % len;

    // 5. 使用标准的 0..len 循环（最大索引到 len-1，绝对不会越界）
    for i in 0..len {
        // 【如果是左旋】：
        let target_idx = (i + k_adapted) % len;
        
        // 【注：如果你其实想要右旋，把上面那行换成下面这行即可】：
        // let target_idx = (i + len - k_adapted) % len;

        print!("{} ", nums[target_idx]);
    }
    println!(); // 最后打印一个换行，让控制台输出更整洁
}
