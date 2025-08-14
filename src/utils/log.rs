/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

// 统一的日志格式 封装
use ::log::LevelFilter;
use env_logger::Env;
use std::sync::OnceLock;

static LOG_CONFIG: OnceLock<LogConfig> = OnceLock::new();

#[derive(Debug)]
pub struct LogConfig {
    pub quiet: bool,
    pub verbose: bool,
}

impl LogConfig {
    pub fn init_logger(quiet: bool, verbose: bool, level: LevelFilter) {
        let config = LogConfig { quiet, verbose };
        LOG_CONFIG.set(config).expect("Logger already initialized");

        // 初始化日志
        env_logger::Builder::from_env(Env::default().default_filter_or(level.to_string()))
            .format(|buf, record| {
                use std::io::Write;
                let level_style = buf.default_level_style(record.level());
                writeln!(
                    buf,
                    "[{} {}{}\x1b[0m {}:{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    level_style,    // 直接使用level_style
                    record.level(), // 重置颜色
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();
    }

    // 打印日志
    pub fn println(msg: &str) {
        if let Some(config) = LOG_CONFIG.get() {
            if !config.quiet {
                println!("{}", msg);
            }
        }
    }
    pub fn print(msg: &str) {
        if let Some(config) = LOG_CONFIG.get() {
            if !config.quiet {
                print!("{}", msg);
            }
        }
    }

    pub fn println_warning(msg: &str) {
        if let Some(config) = LOG_CONFIG.get() {
            if config.quiet {
                return;
            }
            println!("{:>8}utzip warning: {}", "", msg);
        }
    }

    pub fn print_verbose(msg: &str) {
        if let Some(config) = LOG_CONFIG.get() {
            if config.quiet {
                return;
            }
            if config.verbose {
                print!("{}", msg);
            }
        }
    }

    pub fn println_verbose(msg: &str) {
        if let Some(config) = LOG_CONFIG.get() {
            if config.quiet {
                return;
            }
            if config.verbose {
                println!("{}", msg);
            }
        }
    }
}

// 在现有宏下方添加新的宏定义
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        $crate::utils::log::LogConfig::println(&format!($($arg)*));
    }};
}
