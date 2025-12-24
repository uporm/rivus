//! # Rivus Log
//!
//! 一个基于 `tracing` 生态系统的灵活可配置日志库。
//!
//! 该库提供了一个简单的接口来配置日志输出到控制台和文件目标，
//! 支持自定义格式化和轮换选项。
//!
//! ## 特性
//!
//! - 支持控制台和文件日志记录
//! - 可配置的日志级别
//! - 文件输出的自动日志轮换
//! - 配置的 JSON 序列化支持
//! - 非阻塞文件 I/O 以提高性能
//!
//! ## 示例
//!
//! ```rust
//! use rivus_logger::{Logger, LogFile, LogLevel, LogOutput};
//!
//! Logger::new(LogLevel::Info)
//!     .to_console()
//!     .to_file(LogFile::new("./logs", "application")
//!         .with_max_size(10 * 1024 * 1024) // 10MB
//!         .with_max_age(7)) // 7 天
//!     .init();
//!
//! // 现在可以使用 tracing 宏
//! tracing::info!("应用程序已启动");
//! tracing::error!("出现错误");
//! ```

use serde::{Deserialize, Serialize};
use std::io::stdout;
use std::sync::OnceLock;
pub use tracing;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::fmt::format::{DefaultFields, Format, Full};
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Registry};

const DEFAULT_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";
static LOG_GUARD: OnceLock<Vec<WorkerGuard>> = OnceLock::new();

/// 日志级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

/// 日志输出目标枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogOutput {
    Console,
    File,
}

/// 文件日志配置选项。
///
/// 定义基于文件的日志记录设置，包括路径、文件前缀以及可选的
/// 大小和年龄限制用于日志轮换。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogFile {
    /// 存储日志文件的目录路径
    pub path: String,
    /// 日志文件名的前缀
    pub prefix: String,
    /// 轮换前的最大文件大小（字节，可选）
    pub max_size: Option<usize>,
    /// 日志轮换前的最大天数（可选）
    pub max_age: Option<usize>,
}

impl LogFile {
    /// 创建新的文件日志配置
    ///
    /// # 参数
    ///
    /// * `path` - 日志文件目录路径
    /// * `prefix` - 日志文件名前缀
    pub fn new(path: impl Into<String>, prefix: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            prefix: prefix.into(),
            max_size: None,
            max_age: None,
        }
    }

    /// 设置最大文件大小（字节）
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_size = Some(size);
        self
    }

    /// 设置最大保留天数
    pub fn with_max_age(mut self, days: usize) -> Self {
        self.max_age = Some(days);
        self
    }
}

impl Default for LogFile {
    fn default() -> Self {
        Self::new("logs", "app")
    }
}

/// 初始化跟踪子系统的日志配置选项。
///
/// 该结构体定义了设置日志的配置参数，包括日志级别、输出目标
/// 和文件日志设置。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Logger {
    /// 日志级别过滤器
    level: LogLevel,
    /// 输出目标列表
    outputs: Vec<LogOutput>,
    /// 文件日志配置
    file: LogFile,
    /// 时间戳格式（默认为 "%Y-%m-%d %H:%M:%S%.3f"）
    time_format: String,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            outputs: vec![LogOutput::Console],
            file: LogFile::new("logs", "app"),
            time_format: DEFAULT_TIME_FORMAT.to_string(),
        }
    }
}

impl Logger {
    /// 创建默认日志选项
    pub fn new(level: impl Into<LogLevel>) -> Self {
        Self {
            level: level.into(),
            ..Self::default()
        }
    }


    /// 启用控制台输出
    pub fn to_console(mut self) -> Self {
        if !self.outputs.contains(&LogOutput::Console) {
            self.outputs.push(LogOutput::Console);
        }
        self
    }

    /// 启用文件输出
    pub fn to_file(mut self, file: LogFile) -> Self {
        if !self.outputs.contains(&LogOutput::File) {
            self.outputs.push(LogOutput::File);
        }
        self.file = file;
        self
    }

    /// 设置时间戳格式
    ///
    /// 格式字符串遵循 `chrono` 的 `strftime` 语法。
    /// 默认值: "%Y-%m-%d %H:%M:%S%.3f"
    pub fn time_format(mut self, format: impl Into<String>) -> Self {
        self.time_format = format.into();
        self
    }

    /// 初始化日志系统
    pub fn init(self) {
        init(self);
    }
}

/// 创建具有通用格式化选项的基础跟踪层。
///
/// 该函数设置一个标准化层，包含：
/// - 使用 ChronoLocal 的自定义时间戳格式
/// - 启用目标和级别信息
/// - 日志消息的完整格式化
fn create_base_layer<S>(time_format: &str) -> Layer<S, DefaultFields, Format<Full, ChronoLocal>> {
    let timer = ChronoLocal::new(time_format.into());
    fmt::layer()
        .with_timer(timer)
        .with_target(true)
        .with_level(true)
}

fn init(log: Logger) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log.level.as_ref()));
    let registry = Registry::default().with(filter);

    let time_format = &log.time_format;

    let mut layers = Vec::new();
    let mut guards: Vec<WorkerGuard> = Vec::new();

    if log.outputs.is_empty() {
        let console_layer = create_base_layer(time_format).with_writer(stdout).boxed();
        layers.push(console_layer);
    }
    
    for output_target in log.outputs {
        match output_target {
            LogOutput::Console => {
                let console_layer = create_base_layer(time_format).with_writer(stdout).boxed();
                layers.push(console_layer);
            }
            LogOutput::File => {
                let file_config = &log.file;
                let file_appender = rolling::daily(&file_config.path, &file_config.prefix);
                let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
                guards.push(guard);

                let file_layer = create_base_layer(time_format)
                    .with_writer(file_writer)
                    .with_ansi(false)
                    .boxed();
                layers.push(file_layer);
            }
        }
    }

    // 初始化订阅器
    if !layers.is_empty() {
        let subscriber = registry.with(layers);
        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            eprintln!("[错误] 设置全局默认订阅器失败: {}", e);
            return;
        }

        // 存储 guards 以防止过早释放
        if !guards.is_empty() {
            if LOG_GUARD.set(guards).is_err() {
                eprintln!("[错误] 无法设置 LOG_GUARD - 日志可能无法正常工作。");
            }
        }
    } else {
        // 如果没有配置有效输出，回退到控制台
        eprintln!("[错误] 未配置有效的日志输出。默认使用控制台。");
        let default_layer = create_base_layer(time_format).with_writer(stdout);
        let subscriber = registry.with(default_layer);
        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            eprintln!("[错误] 设置回退控制台订阅器失败: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_new_with_level() {
        let logger = Logger::new(LogLevel::Debug);
        assert_eq!(logger.level, LogLevel::Debug);
        // Default outputs should contain Console
        assert!(logger.outputs.contains(&LogOutput::Console));
    }

    #[test]
    fn test_logger_default() {
        let logger = Logger::default();
        assert_eq!(logger.level, LogLevel::Info);
        assert_eq!(logger.outputs.len(), 1);
        assert!(logger.outputs.contains(&LogOutput::Console));
    }

    #[test]
    fn test_logger_outputs() {
        let logger = Logger::new(LogLevel::Info)
            .to_console() // Should stay enabled (default)
            .to_file(LogFile::new("logs", "test"));

        assert!(logger.outputs.contains(&LogOutput::Console));
        assert!(logger.outputs.contains(&LogOutput::File));
        assert_eq!(logger.outputs.len(), 2);
    }

    #[test]
    fn test_log_file_config() {
        let file_config = LogFile::new("test_logs", "test_app")
            .with_max_size(1024)
            .with_max_age(5);

        assert_eq!(file_config.path, "test_logs");
        assert_eq!(file_config.prefix, "test_app");
        assert_eq!(file_config.max_size, Some(1024));
        assert_eq!(file_config.max_age, Some(5));

        let logger = Logger::new(LogLevel::Info).to_file(file_config);
        assert_eq!(logger.file.path, "test_logs");
        assert_eq!(logger.file.max_size, Some(1024));
    }

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from("trace"), LogLevel::Trace);
        assert_eq!(LogLevel::from("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from("Info"), LogLevel::Info);
        assert_eq!(LogLevel::from("warn"), LogLevel::Warn);
        assert_eq!(LogLevel::from("Error"), LogLevel::Error);
        assert_eq!(LogLevel::from("invalid"), LogLevel::Info); // Default fallback

        assert_eq!(LogLevel::Trace.as_ref(), "trace");
        assert_eq!(LogLevel::Error.as_ref(), "error");
    }

    #[test]
    fn test_time_format() {
        let format = "%Y-%m-%d";
        let logger = Logger::new(LogLevel::Info).time_format(format);
        assert_eq!(logger.time_format, format);
    }
}
