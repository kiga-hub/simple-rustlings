// 导入本项目中的 exercise 模块中的 Exercise 和 ExerciseList 类型，并使它们在当前作用域中可用
use crate::exercise::{Exercise, ExerciseList};
use crate::project::RustAnalyzerProject;
use crate::run::{reset, run};

use crate::verify::verify;
// clap 是 Rust 中的一个库，用于解析命令行参数。它提供了一个简单易用的 API，可以帮助开发者快速定义和解析命令行参数，并生成帮助文档和版本信息等。
use clap::{Parser, Subcommand};
// console crate 是一个用于在控制台中输出彩色文本和表情符号的 Rust 库。Emoji 类型是 console crate 中的一个结构体，用于表示一个 Unicode 表情符号
use console::Emoji;
// notify crate 是一个用于监视文件系统事件的 Rust 库。DebouncedEvent 枚举类型是 notify crate 中的一个枚举类型，用于表示文件系统事件
use notify::DebouncedEvent;
// RecommendedWatcher 类型是 notify crate 中的一个结构体，实现了 Watcher trait，并提供了一个推荐的监视器实现
// RecursiveMode 类型是 notify crate 中的一个枚举类型，用于指定监视器是否应该递归地监视子目录
// Watcher 类型是 notify crate 中的一个 trait，用于定义监视文件系统事件的行为
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
// std::ffi::OsStr 是 Rust 标准库中的一个类型，用于表示操作系统原生字符串。
use std::ffi::OsStr;
// std::fs 模块提供了一些与文件系统交互相关的功能，包括文件和目录的创建、删除、重命名、复制、读取和写入等操作。
// std::fs 模块中的函数和方法通常返回 std::io::Result 类型，用于表示操作是否成功
use std::fs;
// std::io 模块提供了与输入输出相关的功能，包括文件读写、标准输入输出、网络通信等
// std::io::prelude 模块中包含了一些常用的 trait，例如 Read、Write、BufRead 等，这些 trait 可以帮助开发者更方便地进行输入输出操作
use std::io::{self, prelude::*};
// std::path 模块提供了一些与文件系统路径相关的功能，包括路径的构建、解析、拼接、比较等。
use std::path::Path;
// 用于导入 std::process 模块中的 Command 和 Stdio 类型，并使它们在当前作用域中可用
// std::process 模块提供了与进程相关的功能，包括创建新进程、与子进程进行交互等
// Command 类型是 std::process 模块中的一个结构体，用于表示要执行的命令及其参数。Command 结构体有一些方法，用于设置命令及其参数
// Stdio 类型是 std::process 模块中的一个枚举类型，用于表示进程的标准输入、输出和错误输出。
// Stdio 枚举类型有三个成员,inherit、piped 和 null，分别表示继承父进程的标准输入、输出和错误输出、创建一个管道和丢弃输入、输出和错误输出定向到空设备
use std::process::{Command, Stdio};
// std::sync::atomic 模块提供了原子类型的支持，包括原子布尔值、原子整数等。
// Ordering 类型是 std::sync::atomic 模块中的一个枚举类型，用于表示原子操作的内存顺序。
use std::sync::atomic::{AtomicBool, Ordering};
// std::sync::mpsc 模块提供了多个生产者、单个消费者（MSPC）通道的支持，用于在多个线程之间传递消息。
// channel 函数是 std::sync::mpsc 模块中的一个函数，用于创建一个新的 MSPC 通道。
// RecvTimeoutError 类型是 std::sync::mpsc 模块中的一个枚举类型，用于表示从 MSPC 通道接收消息时可能出现的超时错误。
use std::sync::mpsc::{channel, RecvTimeoutError};
// std::sync 模块提供了多种并发原语的支持，包括原子类型、互斥锁、条件变量等。
// Arc 类型是 std::sync 模块中的一个结构体，用于表示原子引用计数。
// Mutex 类型是 std::sync 模块中的一个结构体，用于表示互斥锁。
use std::sync::{Arc, Mutex};
// std::thread 模块提供了线程的支持，包括创建线程、等待线程完成、线程同步等。
use std::thread;
// std::time 模块提供了时间相关的支持，包括计时器、时间间隔等。Duration 类型是 std::time 模块中的一个结构体，用于表示时间间隔。
use std::time::Duration;

// #[macro_use] 是一个属性宏，用于在 Rust 中导入宏并使其在当前作用域中可用。
// 定义在 ui 模块中的宏可以在当前模块中使用，而不需要重新定义
#[macro_use]
mod ui;
mod exercise;
mod project;
mod run;
mod verify;

/// 编译器为这个结构体自动生成命令行参数解析器。
#[derive(Parser)]
/// 诉编译器这个结构体是一个命令行程序，并且它有一个名为 version 的子命令
#[command(version)]
struct Args {
    /// Show outputs from the test exercises
    /// #[arg(long)] 属性，表示它是一个长选项。
    #[arg(long)]
    nocapture: bool,
    /// #[command(subcommand)] 属性，表示它是一个子命令。
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Verify all exercises according to the recommended order
    Verify,
    /// Rerun `verify` when files were edited
    Watch {
        /// Show hints on success
        #[arg(long)]
        success_hints: bool,
    },
    /// Run/Test a single exercise
    Run {
        /// The name of the exercise
        name: String,
    },
    /// Reset a single exercise using "git stash -- filename"
    Reset {
        /// The name of the exercise
        name: String,
    },
    /// Return a hint for the given exercise
    Hint {
        /// The name of the exercise
        name: String,
    },
    /// List the exercises available in kiga
    List {
        /// Show only the paths of the exercises
        #[arg(short, long)]
        paths: bool,
        /// Show only the names of the exercises
        #[arg(short, long)]
        names: bool,
        /// Provide a string to match exercise names.
        /// Comma separated patterns are accepted
        #[arg(short, long)]
        filter: Option<String>,
        /// Display only exercises not yet solved
        #[arg(short, long)]
        unsolved: bool,
        /// Display only exercises that have been solved
        #[arg(short, long)]
        solved: bool,
    },
    /// Enable rust-analyzer for exercises
    Lsp,
}

fn main() {
    // 解析命令行参数
    let args = Args::parse();

    // 如果没有提供子命令，则打印欢迎信息
    if args.command.is_none() {
        println!("\n{WELCOME}\n");
    }

    // 如果当前目录下没有 info.toml 文件，则打印错误信息并退出程序
    if !Path::new("info.toml").exists() {
        println!(
            "{} must be run from the kiga directory",
            std::env::current_exe().unwrap().to_str().unwrap()
        );
        println!("Try `cd kiga/`!");
        std::process::exit(1);
    }

    // 如果当前目录下没有 exercises 目录，则打印错误信息并退出程序
    if !rustc_exists() {
        println!("We cannot find `rustc`.");
        println!("Try running `rustc --version` to diagnose your problem.");
        println!("For instructions on how to install Rust, check the README.");
        std::process::exit(1);
    }

    // 从 info.toml 文件中读取练习列表
    let toml_str = &fs::read_to_string("info.toml").unwrap();
    // 将练习列表解析为 ExerciseList 类型
    let exercises = toml::from_str::<ExerciseList>(toml_str).unwrap().exercises;
    // 如果没有提供子命令，则打印练习列表并退出程序,verbose 为 true 表示打印练习列表
    let verbose = args.nocapture;
    let command = args.command.unwrap_or_else(|| {
        println!("{DEFAULT_OUT}\n");
        std::process::exit(0);
    });

    // 根据提供的子命令执行相应的操作
    match command {
        // 如果提供的子命令是 List，则打印练习列表
        Subcommands::List {
            paths,
            names,
            filter,
            unsolved,
            solved,
        } => {
            //
            if !paths && !names {
                println!("{:<17}\t{:<46}\t{:<7}", "Name", "Path", "Status");
            }
            // 统计已完成的练习数量
            let mut exercises_done: u16 = 0;
            // 将 filter 转换为小写字母
            let filters = filter.clone().unwrap_or_default().to_lowercase();
            // 遍历练习列表中的每一个练习
            exercises.iter().for_each(|e| {
                // 将练习的路径转换为字符串
                let fname = format!("{}", e.path.display());
                // 如果练习的名称或路径中包含 filter 中的字符串，则将 filter_cond 设置为 true，否则设置为 false
                let filter_cond = filters
                    .split(',')
                    .filter(|f| !f.trim().is_empty())
                    .any(|f| e.name.contains(f) || fname.contains(f));
                // 如果练习已完成，则将 status 设置为 Done，否则设置为 Pending
                let status = if e.looks_done() {
                    exercises_done += 1;
                    "Done"
                } else {
                    "Pending"
                };
                // 如果练习已完成，则将 solved 设置为 true，否则设置为 false
                let solve_cond = {
                    (e.looks_done() && solved)
                        || (!e.looks_done() && unsolved)
                        || (!solved && !unsolved)
                };
                // 如果练习已完成且 filter_cond 为 true，则打印练习的名称、路径和状态
                if solve_cond && (filter_cond || filter.is_none()) {
                    let line = if paths {
                        format!("{fname}\n")
                    } else if names {
                        format!("{}\n", e.name)
                    } else {
                        format!("{:<17}\t{fname:<46}\t{status:<7}\n", e.name)
                    };
                    // Somehow using println! leads to the binary panicking
                    // when its output is piped.
                    // So, we're handling a Broken Pipe error and exiting with 0 anyway
                    let stdout = std::io::stdout();
                    {
                        // 临时获取 stdout 的锁
                        let mut handle = stdout.lock();
                        // 将 line 写入 stdout
                        handle.write_all(line.as_bytes()).unwrap_or_else(|e| {
                            match e.kind() {
                                std::io::ErrorKind::BrokenPipe => std::process::exit(0),
                                _ => std::process::exit(1),
                            };
                        });
                    }
                }
            });
            // 打印练习完成的百分比
            let percentage_progress = exercises_done as f32 / exercises.len() as f32 * 100.0;
            println!(
                "Progress: You completed {} / {} exercises ({:.1} %).",
                exercises_done,
                exercises.len(),
                percentage_progress
            );
            std::process::exit(0);
        }

        // 如果提供的子命令是 Run，则运行指定的练习
        Subcommands::Run { name } => {
            let exercise = find_exercise(&name, &exercises);

            run(exercise, verbose).unwrap_or_else(|_| std::process::exit(1));
        }

        // 如果提供的子命令是 Reset，则重置指定的练习
        Subcommands::Reset { name } => {
            let exercise = find_exercise(&name, &exercises);

            reset(exercise).unwrap_or_else(|_| std::process::exit(1));
        }

        // 如果提供的子命令是 Hint，则打印指定练习的提示
        Subcommands::Hint { name } => {
            let exercise = find_exercise(&name, &exercises);

            println!("{}", exercise.hint);
        }

        // 如果提供的子命令是 Verify，则验证所有练习
        Subcommands::Verify => {
            verify(&exercises, (0, exercises.len()), verbose, false)
                .unwrap_or_else(|_| std::process::exit(1));
        }

        // 如果提供的子命令是 Lsp，则生成 rust-project.json 文件
        Subcommands::Lsp => {
            let mut project = RustAnalyzerProject::new();
            project
                .get_sysroot_src()
                .expect("Couldn't find toolchain path, do you have `rustc` installed?");
            project
                .exercises_to_json()
                .expect("Couldn't parse kiga exercises files");

            if project.crates.is_empty() {
                println!("Failed find any exercises, make sure you're in the `kiga` folder");
            } else if project.write_to_disk().is_err() {
                println!("Failed to write rust-project.json to disk for rust-analyzer");
            } else {
                println!("Successfully generated rust-project.json");
                println!("rust-analyzer will now parse exercises, restart your language server or editor")
            }
        }

        // 如果提供的子命令是 Watch，则启动监视器
        Subcommands::Watch { success_hints } => match watch(&exercises, verbose, success_hints) {
            Err(e) => {
                println!(
                    "Error: Could not watch your progress. Error message was {:?}.",
                    e
                );
                println!("Most likely you've run out of disk space or your 'inotify limit' has been reached.");
                std::process::exit(1);
            }
            Ok(WatchStatus::Finished) => {
                println!(
                    "{emoji} All exercises completed! {emoji}",
                    emoji = Emoji("🎉", "★")
                );
                println!("\n{FENISH_LINE}\n");
            }
            Ok(WatchStatus::Unfinished) => {
                println!("We hope you're enjoying learning about Rust!");
                println!("If you want to continue working on the exercises at a later point, you can simply run `kiga watch` again");
            }
        },
    }
}

// spawn_watch_shell 函数用于启动一个新的线程，用于监听用户输入的命令
fn spawn_watch_shell(
    // failed_exercise_hint 是一个 Arc<Mutex<Option<String>>> 类型的变量，用于存储当前练习的提示
    failed_exercise_hint: &Arc<Mutex<Option<String>>>,
    // should_quit 是一个 Arc<AtomicBool> 类型的变量，用于表示是否退出程序
    should_quit: Arc<AtomicBool>,
) {
    // 将 failed_exercise_hint 和 should_quit 移动到新线程中
    let failed_exercise_hint = Arc::clone(failed_exercise_hint);
    println!("Welcome to watch mode! You can type 'help' to get an overview of the commands you can use here.");
    // 启动一个新的线程，用于监听用户输入的命令
    thread::spawn(move || loop {
        // 创建一个新的字符串变量，用于存储用户输入的命令
        let mut input = String::new();
        // 从标准输入中读取用户输入的命令
        match io::stdin().read_line(&mut input) {
            // 如果读取成功，则执行相应的命令
            Ok(_) => {
                // 去掉命令中的空白字符
                let input = input.trim();
                // 如果命令是 hint，则打印当前练习的提示
                if input == "hint" {
                    if let Some(hint) = &*failed_exercise_hint.lock().unwrap() {
                        println!("{hint}");
                    }
                    // 如果命令是 clear，则清空终端
                } else if input == "clear" {
                    println!("\x1B[2J\x1B[1;1H");
                    // 如果命令是 quit，则退出程序
                } else if input.eq("quit") {
                    // 将 should_quit 设置为 true
                    should_quit.store(true, Ordering::SeqCst);
                    println!("Bye!");
                    // 如果命令是 help，则打印帮助信息
                } else if input.eq("help") {
                    println!("Commands available to you in watch mode:");
                    println!("  hint   - prints the current exercise's hint");
                    println!("  clear  - clears the screen");
                    println!("  quit   - quits watch mode");
                    println!("  !<cmd> - executes a command, like `!rustc --explain E0381`");
                    println!("  help   - displays this help message");
                    println!();
                    println!("Watch mode automatically re-evaluates the current exercise");
                    println!("when you edit a file's contents.")
                    // 如果命令以 ! 开头，则执行相应的命令
                } else if let Some(cmd) = input.strip_prefix('!') {
                    // 将命令按空白字符分割为多个部分
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    // 如果命令为空，则打印错误信息
                    if parts.is_empty() {
                        println!("no command provided");
                        // 否则，执行相应的命令
                        // Command::new(parts[0]) 用于创建一个新的命令
                        // args(&parts[1..]) 用于设置命令的参数
                        // status() 用于执行命令并返回执行结果
                    } else if let Err(e) = Command::new(parts[0]).args(&parts[1..]).status() {
                        println!("failed to execute command `{}`: {}", cmd, e);
                    }
                } else {
                    println!("unknown command: {input}");
                }
            }
            // 如果读取失败，则打印错误信息
            Err(error) => println!("error reading command: {error}"),
        }
    });
}

// find_exercise 函数用于在练习列表中查找指定名称的练习
fn find_exercise<'a>(name: &str, exercises: &'a [Exercise]) -> &'a Exercise {
    // 如果提供的名称是 next，则查找第一个未完成的练习
    if name.eq("next") {
        exercises
            .iter()
            .find(|e| !e.looks_done())
            .unwrap_or_else(|| {
                println!("🎉 Congratulations! You have done all the exercises!");
                println!("🔚 There are no more exercises to do next!");
                std::process::exit(1)
            })
    } else {
        exercises
            .iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| {
                println!("No exercise found for '{name}'!");
                std::process::exit(1)
            })
    }
}

// WatchStatus 枚举类型用于表示监视器的状态
enum WatchStatus {
    Finished,
    Unfinished,
}

// watch 函数用于启动一个监视器，用于监视文件系统事件
fn watch(
    // exercises 是一个 &[Exercise] 类型的变量，用于存储练习列表
    exercises: &[Exercise],
    // verbose 是一个 bool 类型的变量，用于表示是否打印练习列表
    verbose: bool,
    // success_hints 是一个 bool 类型的变量，用于表示是否在练习完成时打印提示
    success_hints: bool,
    // watch 函数返回一个 Result<WatchStatus> 类型的结果
) -> notify::Result<WatchStatus> {
    /* Clears the terminal with an ANSI escape code.
    Works in UNIX and newer Windows terminals. */
    fn clear_screen() {
        println!("\x1Bc");
    }

    // 创建一个新的通道，用于在监视器和主线程之间传递消息
    let (tx, rx) = channel();
    // 创建一个新的原子布尔值，用于表示是否退出程序
    let should_quit = Arc::new(AtomicBool::new(false));

    // 创建一个新的监视器，用于监视文件系统事件
    // tx 是一个 Sender<DebouncedEvent> 类型的变量，用于向监视器发送消息
    // Duration::from_secs(1) 表示监视器每隔 1 秒检查一次文件系统事件
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
    // 将当前目录下的所有文件和子目录添加到监视器中
    watcher.watch(Path::new("./exercises"), RecursiveMode::Recursive)?;

    // 清空终端
    clear_screen();

    // to_owned_hint 函数用于将 Exercise 类型的变量转换为 String 类型的变量
    let to_owned_hint = |t: &Exercise| t.hint.to_owned();
    // failed_exercise_hint 是一个 Arc<Mutex<Option<String>>> 类型的变量，用于存储当前练习的提示
    let failed_exercise_hint = match verify(
        // exercises.iter() 用于创建一个迭代器，用于遍历练习列表
        exercises.iter(),
        (0, exercises.len()),
        verbose,
        success_hints,
    ) {
        Ok(_) => return Ok(WatchStatus::Finished),
        Err(exercise) => Arc::new(Mutex::new(Some(to_owned_hint(exercise)))),
    };

    // 启动一个新的线程，用于监听用户输入的命令
    spawn_watch_shell(&failed_exercise_hint, Arc::clone(&should_quit));
    loop {
        // 接收来自监视器的消息
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => match event {
                // 如果接收到的消息是 Create、Chmod 或 Write，则检查是否有练习完成
                DebouncedEvent::Create(b) | DebouncedEvent::Chmod(b) | DebouncedEvent::Write(b) => {
                    // 如果文件的扩展名是 rs，则检查是否有练习完成
                    if b.extension() == Some(OsStr::new("rs")) && b.exists() {
                        // 将文件的路径转换为绝对路径
                        let filepath = b.as_path().canonicalize().unwrap();
                        // 从练习列表中查找指定路径的练习
                        let pending_exercises = exercises
                            .iter()
                            // 如果练习已完成，则将其从练习列表中移除
                            .find(|e| filepath.ends_with(&e.path))
                            .into_iter()
                            .chain(
                                exercises
                                    .iter()
                                    .filter(|e| !e.looks_done() && !filepath.ends_with(&e.path)),
                            );
                            // 统计已完成的练习数量
                        let num_done = exercises.iter().filter(|e| e.looks_done()).count();
                        clear_screen();
                        match verify(
                            pending_exercises,
                            (num_done, exercises.len()),
                            verbose,
                            success_hints,
                        ) {
                            Ok(_) => return Ok(WatchStatus::Finished),
                            Err(exercise) => {
                                let mut failed_exercise_hint = failed_exercise_hint.lock().unwrap();
                                *failed_exercise_hint = Some(to_owned_hint(exercise));
                            }
                        }
                    }
                }
                _ => {}
            },
            // 如果接收到的消息是超时，则继续循环
            Err(RecvTimeoutError::Timeout) => {
                // the timeout expired, just check the `should_quit` variable below then loop again
            }
            Err(e) => println!("watch error: {e:?}"),
        }
        // Check if we need to exit
        if should_quit.load(Ordering::SeqCst) {
            return Ok(WatchStatus::Unfinished);
        }
    }
}

// rustc_exists 函数用于检查是否安装了 Rust 编译器
fn rustc_exists() -> bool {
    // Command::new("rustc") 用于创建一个新的命令
    Command::new("rustc")
        .args(["--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .and_then(|mut child| child.wait())
        .map(|status| status.success())
        .unwrap_or(false)
}

const DEFAULT_OUT: &str = r#"Thanks for installing kiga!"#;

const FENISH_LINE: &str = r"+----------------------------------------------------+
|          You made it to the Fe-nish line!          |";

const WELCOME: &str = r"Welcome To KIGA! ";
