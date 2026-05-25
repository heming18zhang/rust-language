use std::io;

fn main() {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    let s = s.trim();
		let mut stack: Vec<char> = vec![];
  	for ch in s.chars() {
    	match ch {
      	// 如果是左括号，直接压栈（Push）
            '(' | '[' | '{' => stack.push(ch),
            
            // 如果是右括号，弹出栈顶元素进行比对
            ')' => if stack.pop() != Some('(') { break; },
            ']' => if stack.pop() != Some('[') { break; },
            '}' => if stack.pop() != Some('{') { break; },
            
            // 过滤掉其他非括号噪声（如果有的话）
            _ => {}
        }
        
      }
   
    // Check if the brackets are balanced
	println!("{}", (if stack.is_empty() {"Yes"} else {"No"}) );
}
