// 导入本低项目中的 exercise 模块中的 Exercise 和 ExerciseList 类型，并使它们在当前作用域中可用
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
    let args = Args::parse();

    if args.command.is_none() {
        println!("\n{WELCOME}\n");
    }

    if !Path::new("info.toml").exists() {
        println!(
            "{} must be run from the kiga directory",
            std::env::current_exe().unwrap().to_str().unwrap()
        );
        println!("Try `cd kiga/`!");
        std::process::exit(1);
    }

    if !rustc_exists() {
        println!("We cannot find `rustc`.");
        println!("Try running `rustc --version` to diagnose your problem.");
        println!("For instructions on how to install Rust, check the README.");
        std::process::exit(1);
    }

    let toml_str = &fs::read_to_string("info.toml").unwrap();
    let exercises = toml::from_str::<ExerciseList>(toml_str).unwrap().exercises;
    let verbose = args.nocapture;

    let command = args.command.unwrap_or_else(|| {
        println!("{DEFAULT_OUT}\n");
        std::process::exit(0);
    });

    match command {
        Subcommands::List {
            paths,
            names,
            filter,
            unsolved,
            solved,
        } => {
            if !paths && !names {
                println!("{:<17}\t{:<46}\t{:<7}", "Name", "Path", "Status");
            }
            let mut exercises_done: u16 = 0;
            let filters = filter.clone().unwrap_or_default().to_lowercase();
            exercises.iter().for_each(|e| {
                let fname = format!("{}", e.path.display());
                let filter_cond = filters
                    .split(',')
                    .filter(|f| !f.trim().is_empty())
                    .any(|f| e.name.contains(f) || fname.contains(f));
                let status = if e.looks_done() {
                    exercises_done += 1;
                    "Done"
                } else {
                    "Pending"
                };
                let solve_cond = {
                    (e.looks_done() && solved)
                        || (!e.looks_done() && unsolved)
                        || (!solved && !unsolved)
                };
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
                        let mut handle = stdout.lock();
                        handle.write_all(line.as_bytes()).unwrap_or_else(|e| {
                            match e.kind() {
                                std::io::ErrorKind::BrokenPipe => std::process::exit(0),
                                _ => std::process::exit(1),
                            };
                        });
                    }
                }
            });
            let percentage_progress = exercises_done as f32 / exercises.len() as f32 * 100.0;
            println!(
                "Progress: You completed {} / {} exercises ({:.1} %).",
                exercises_done,
                exercises.len(),
                percentage_progress
            );
            std::process::exit(0);
        }

        Subcommands::Run { name } => {
            let exercise = find_exercise(&name, &exercises);

            run(exercise, verbose).unwrap_or_else(|_| std::process::exit(1));
        }

        Subcommands::Reset { name } => {
            let exercise = find_exercise(&name, &exercises);

            reset(exercise).unwrap_or_else(|_| std::process::exit(1));
        }

        Subcommands::Hint { name } => {
            let exercise = find_exercise(&name, &exercises);

            println!("{}", exercise.hint);
        }

        Subcommands::Verify => {
            verify(&exercises, (0, exercises.len()), verbose, false)
                .unwrap_or_else(|_| std::process::exit(1));
        }

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

fn spawn_watch_shell(
    failed_exercise_hint: &Arc<Mutex<Option<String>>>,
    should_quit: Arc<AtomicBool>,
) {
    let failed_exercise_hint = Arc::clone(failed_exercise_hint);
    println!("Welcome to watch mode! You can type 'help' to get an overview of the commands you can use here.");
    thread::spawn(move || loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input == "hint" {
                    if let Some(hint) = &*failed_exercise_hint.lock().unwrap() {
                        println!("{hint}");
                    }
                } else if input == "clear" {
                    println!("\x1B[2J\x1B[1;1H");
                } else if input.eq("quit") {
                    should_quit.store(true, Ordering::SeqCst);
                    println!("Bye!");
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
                } else if let Some(cmd) = input.strip_prefix('!') {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.is_empty() {
                        println!("no command provided");
                    } else if let Err(e) = Command::new(parts[0]).args(&parts[1..]).status() {
                        println!("failed to execute command `{}`: {}", cmd, e);
                    }
                } else {
                    println!("unknown command: {input}");
                }
            }
            Err(error) => println!("error reading command: {error}"),
        }
    });
}

fn find_exercise<'a>(name: &str, exercises: &'a [Exercise]) -> &'a Exercise {
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

enum WatchStatus {
    Finished,
    Unfinished,
}

fn watch(
    exercises: &[Exercise],
    verbose: bool,
    success_hints: bool,
) -> notify::Result<WatchStatus> {
    /* Clears the terminal with an ANSI escape code.
    Works in UNIX and newer Windows terminals. */
    fn clear_screen() {
        println!("\x1Bc");
    }

    let (tx, rx) = channel();
    let should_quit = Arc::new(AtomicBool::new(false));

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
    watcher.watch(Path::new("./exercises"), RecursiveMode::Recursive)?;

    clear_screen();

    let to_owned_hint = |t: &Exercise| t.hint.to_owned();
    let failed_exercise_hint = match verify(
        exercises.iter(),
        (0, exercises.len()),
        verbose,
        success_hints,
    ) {
        Ok(_) => return Ok(WatchStatus::Finished),
        Err(exercise) => Arc::new(Mutex::new(Some(to_owned_hint(exercise)))),
    };
    spawn_watch_shell(&failed_exercise_hint, Arc::clone(&should_quit));
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => match event {
                DebouncedEvent::Create(b) | DebouncedEvent::Chmod(b) | DebouncedEvent::Write(b) => {
                    if b.extension() == Some(OsStr::new("rs")) && b.exists() {
                        let filepath = b.as_path().canonicalize().unwrap();
                        let pending_exercises = exercises
                            .iter()
                            .find(|e| filepath.ends_with(&e.path))
                            .into_iter()
                            .chain(
                                exercises
                                    .iter()
                                    .filter(|e| !e.looks_done() && !filepath.ends_with(&e.path)),
                            );
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

fn rustc_exists() -> bool {
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

const DEFAULT_OUT: &str = r#"Thanks for installing kiga!

Is this your first time? Don't worry, kiga was made for beginners! We are
going to teach you a lot of things about Rust, but before we can get
started, here's a couple of notes about how kiga operates:

1. The central concept behind kiga is that you solve exercises. These
   exercises usually have some sort of syntax error in them, which will cause
   them to fail compilation or testing. Sometimes there's a logic error instead
   of a syntax error. No matter what error, it's your job to find it and fix it!
   You'll know when you fixed it because then, the exercise will compile and
   kiga will be able to move on to the next exercise.
2. If you run kiga in watch mode (which we recommend), it'll automatically
   start with the first exercise. Don't get confused by an error message popping
   up as soon as you run kiga! This is part of the exercise that you're
   supposed to solve, so open the exercise file in an editor and start your
   detective work!
3. If you're stuck on an exercise, there is a helpful hint you can view by typing
   'hint' (in watch mode), or running `kiga hint exercise_name`.
4. If an exercise doesn't make sense to you, feel free to open an issue on GitHub!
   (https://github.com/rust-lang/kiga/issues/new). We look at every issue,
   and sometimes, other learners do too so you can help each other out!
5. If you want to use `rust-analyzer` with exercises, which provides features like
   autocompletion, run the command `kiga lsp`.

Got all that? Great! To get started, run `kiga watch` in order to get the first
exercise. Make sure to have your editor open!"#;

const FENISH_LINE: &str = r"+----------------------------------------------------+
|          You made it to the Fe-nish line!          |
We hope you enjoyed learning about the various aspects of Rust!
If you noticed any issues, please don't hesitate to report them to our repo.
You can also contribute your own exercises to help the greater community!

Before reporting an issue or contributing, please read our guidelines:
https://github.com/rust-lang/kiga/blob/main/CONTRIBUTING.md";

const WELCOME: &str = r"Welcome To KIGA! ";
