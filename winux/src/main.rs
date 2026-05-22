use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;

// 使用 #[derive(Parser)] 让 Cli 结构体具备自动解析命令行参数的能力
#[derive(Parser)]
#[command(name = "winux", version = "2.0", about = "在 Windows 上运行的 Linux 核心命令大集合")]
struct Cli {
    // 嵌套子命令解析
    #[command(subcommand)]
    command: Commands,
}

// 定义一个枚举，枚举的每一个变体都对应一个 Linux 命令
#[derive(Subcommand)]
enum Commands {
    // === 原有命令保持并兼容 ===
    #[command(about = "列出当前目录下的文件和文件夹 (ls)")]
    Ls {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    #[command(about = "显示当前工作目录的绝对路径 (pwd)")]
    Pwd,
    #[command(about = "连接文件并打印到标准输出 (cat)")]
    Cat {
        file: PathBuf,
    },
    #[command(about = "创建空文件或更新文件修改时间 (touch)")]
    Touch {
        file: PathBuf,
    },
    #[command(about = "删除文件或目录 (rm)")]
    Rm {
        file: PathBuf,
        #[arg(short, long, help = "递归删除目录及其内容")]
        recursive: bool,
    },
    #[command(about = "创建新目录 (mkdir)")]
    Mkdir {
        dir: PathBuf,
    },
    #[command(about = "清除屏幕终端 (clear)")]
    Clear,

    // === 新增扩容的 Linux 命令 ===
    #[command(about = "复制文件或目录 (cp)")]
    Cp {
        source: PathBuf,
        destination: PathBuf,
    },
    #[command(about = "移动或重命名文件/目录 (mv)")]
    Mv {
        source: PathBuf,
        destination: PathBuf,
    },
    #[command(about = "显示当前登录的用户名 (whoami)")]
    Whoami,
    #[command(about = "显示系统的当前日期和时间 (date)")]
    Date,
    #[command(about = "显示文件的开头部分内容 (head)")]
    Head {
        file: PathBuf,
        #[arg(short, long, default_value = "10", help = "显示开头的行数")]
        lines: usize,
    },
    #[command(about = "显示文件的结尾部分内容 (tail)")]
    Tail {
        file: PathBuf,
        #[arg(short, long, default_value = "10", help = "显示结尾的行数")]
        lines: usize,
    },
    #[command(about = "打印系统信息 (uname)")]
    Uname {
        #[arg(short, long, help = "打印系统所有相关信息")]
        all: bool,
    },
}

fn main() {
    // 解析用户在命令行输入的参数
    let cli = Cli::parse();

    // 根据解析出来的命令进行业务分发
    match &cli.command {
        // 1. ls 命令逻辑
        Commands::Ls { path } => {
            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        if entry.path().is_dir() {
                            println!("{}/", name); // 文件夹加斜杠区分
                        } else {
                            println!("{}", name);
                        }
                    }
                }
                Err(e) => eprintln!("ls 失败: {}", e),
            }
        }
        
        // 2. pwd 命令逻辑
        Commands::Pwd => {
            match std::env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => eprintln!("pwd 失败: {}", e),
            }
        }

        // 3. cat 命令逻辑
        Commands::Cat { file } => {
            match fs::read_to_string(file) {
                Ok(content) => print!("{}", content),
                Err(e) => eprintln!("cat 失败: {}", e),
            }
        }

        // 4. touch 命令逻辑
        Commands::Touch { file } => {
            match File::create(file) {
                Ok(_) => {},
                Err(e) => eprintln!("touch 失败: {}", e),
            }
        }

        // 5. rm 命令逻辑
        Commands::Rm { file, recursive } => {
            if file.is_dir() {
                if *recursive {
                    match fs::remove_dir_all(file) {
                        Ok(_) => {},
                        Err(e) => eprintln!("rm -r 失败: {}", e),
                    }
                } else {
                    eprintln!("rm: 无法删除 '{}': 是一个目录 (请使用 -r 参数)", file.display());
                }
            } else {
                match fs::remove_file(file) {
                    Ok(_) => {},
                    Err(e) => eprintln!("rm 失败: {}", e),
                }
            }
        }

        // 6. mkdir 命令逻辑
        Commands::Mkdir { dir } => {
            match fs::create_dir_all(dir) {
                Ok(_) => {},
                Err(e) => eprintln!("mkdir 失败: {}", e),
            }
        }

        // 7. clear 命令逻辑
        Commands::Clear => {
            // 调用 Windows 底层 cmd 的 cls 指令来实现终端清屏
            let _ = Command::new("cmd").args(["/C", "cls"]).status();
        }

        // 8. 新增: cp (复制) 命令逻辑
        Commands::Cp { source, destination } => {
            // fs::copy 封装了 Windows 的 CopyFileW API
            match fs::copy(source, destination) {
                Ok(_) => {},
                Err(e) => eprintln!("cp 失败: {}", e),
            }
        }

        // 9. 新增: mv (移动/重命名) 命令逻辑
        Commands::Mv { source, destination } => {
            // fs::rename 对应 Windows 的 MoveFileExW API，可同时实现移动或原地改名
            match fs::rename(source, destination) {
                Ok(_) => {},
                Err(e) => eprintln!("mv 失败: {}", e),
            }
        }

        // 10. 新增: whoami 命令逻辑
        Commands::Whoami => {
            // 在 Windows 环境下，通过读取环境变量 USERNAME 来获取当前登录的用户主体
            match std::env::var("USERNAME") {
                Ok(username) => println!("{}", username),
                Err(_) => eprintln!("无法获取当前用户名"),
            }
        }

        // 11. 新增: date 命令逻辑
        Commands::Date => {
            // 使用标准库的 SystemTime 获取当前系统时间戳
            let now = std::time::SystemTime::now();
            // 将其转换为 Windows 的本地时间格式或标准格式。
            // 为了保持零依赖不引入 chrono 库，这里直接调用本地系统命令 `date /T` 和 `time /T` 的组合
            let output = Command::new("cmd").args(["/C", "echo %DATE% %TIME%"]).output();
            match output {
                Ok(out) => {
                    let date_str = String::from_utf8_lossy(&out.stdout);
                    print!("{}", date_str);
                }
                Err(_) => {
                    // 如果系统调用失败，作为保底输出标准时间戳
                    println!("{:?}", now);
                }
            }
        }

        // 12. 新增: head 命令逻辑
        Commands::Head { file, lines } => {
            match File::open(file) {
                Ok(f) => {
                    // 使用 BufReader 按行缓冲读取文件，避免大文件直接加载进内存导致卡死
                    let reader = BufReader::new(f);
                    // take(*lines) 限制只获取前 N 行
                    for line in reader.lines().take(*lines) {
                        match line {
                            Ok(text) => println!("{}", text),
                            Err(e) => { eprintln!("读取行失败: {}", e); break; }
                        }
                    }
                }
                Err(e) => eprintln!("head 失败: {}", e),
            }
        }

        // 13. 新增: tail 命令逻辑
        Commands::Tail { file, lines } => {
            match File::open(file) {
                Ok(f) => {
                    let reader = BufReader::new(f);
                    // 将所有行读入一个 Vector 数组中（适用于常规文本查看）
                    let all_lines: Vec<io::Result<String>> = reader.lines().collect();
                    let total = all_lines.len();
                    // 计算从哪一行开始截取（总行数减去需要的行数，如果不足 0 则从 0 开始）
                    let start = if total > *lines { total - *lines } else { 0 };
                    
                    for line in all_lines.into_iter().skip(start) {
                        match line {
                            Ok(text) => println!("{}", text),
                            Err(e) => eprintln!("读取行失败: {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("tail 失败: {}", e),
            }
        }

        // 14. 新增: uname 命令逻辑
        Commands::Uname { all } => {
            // std::env::consts::OS 在 Windows 上编译时会固定返回 "windows"
            let os = std::env::consts::OS;
            // std::env::consts::ARCH 返回当前 CPU 架构（如 "x86_64" 或 "aarch64"）
            let arch = std::env::consts::ARCH;
            
            if *all {
                // 模拟 Linux 的 uname -a 行为，打印系统类型、主机名和硬件架构
                let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown_host".to_string());
                println!("{} {} 1.0.0-winux-generic {}", os, hostname, arch);
            } else {
                // 默认直接打印操作系统内核大类（Windows）
                println!("{}", os);
            }
        }
    }
}