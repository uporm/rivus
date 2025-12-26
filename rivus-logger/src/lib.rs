use log::error;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    Layer, Registry,
    fmt::{self, time::ChronoLocal},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

const DEFAULT_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";
const DEFAULT_LOG_DIR: &str = "./logs";
const DEFAULT_FILE_PREFIX: &str = "app.log";
const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(3600);

/// 日志配置构建器
///
/// 用于配置和初始化日志系统，支持控制台输出和文件滚动输出，
/// 并提供自动清理旧日志文件的功能。
pub struct LoggerConfig {
    /// 日志文件前缀 (实际文件名会包含日期，如 app.log.2023-10-01)
    file_prefix: String,
    /// 日志存储目录
    log_dir: PathBuf,
    /// 时间格式字符串 (基于 Chrono 格式)
    time_format: String,
    /// 日志级别
    level: String,
    /// 是否启用控制台输出
    console: bool,
    /// 是否启用文件输出
    file: bool,
    /// 保留的最大日志文件数量 (用于自动清理)
    max_files: Option<i16>,
    /// 清理任务检查间隔
    cleanup_interval: Duration,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            file_prefix: DEFAULT_FILE_PREFIX.to_string(),
            log_dir: PathBuf::from(DEFAULT_LOG_DIR),
            time_format: DEFAULT_TIME_FORMAT.to_string(),
            level: "INFO".to_string(),
            console: true,
            file: true,
            max_files: None,
            cleanup_interval: DEFAULT_CLEANUP_INTERVAL,
        }
    }
}

impl LoggerConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置日志文件前缀
    ///
    /// 滚动日志文件将以此为前缀，并附加日期后缀。
    pub fn file_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.file_prefix = prefix.into();
        self
    }

    /// 设置日志存储目录
    pub fn log_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.log_dir = dir.into();
        self
    }

    /// 设置日志时间戳格式
    ///
    /// 格式参考 chrono::format::strftime
    pub fn time_format(mut self, format: impl Into<String>) -> Self {
        self.time_format = format.into();
        self
    }

    /// 设置全局日志级别
    pub fn level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    /// 启用或禁用控制台日志输出
    pub fn enable_console(mut self, enable: bool) -> Self {
        self.console = enable;
        self
    }

    /// 启用或禁用文件日志输出
    pub fn enable_file(mut self, enable: bool) -> Self {
        self.file = enable;
        self
    }

    /// 设置保留的最大日志文件数量
    ///
    /// 超过此数量的旧日志文件将被自动删除。
    pub fn max_files(mut self, count: i16) -> Self {
        self.max_files = Some(count);
        self
    }

    /// 设置日志清理任务的检查间隔
    pub fn cleanup_interval(mut self, interval: Duration) -> Self {
        self.cleanup_interval = interval;
        self
    }

    /// 初始化日志系统
    ///
    /// 该方法会消耗配置对象，注册全局 tracing subscriber，并启动清理任务（如果配置了 max_files）。
    /// 返回的 `Option<WorkerGuard>` 必须被持有，以确保异步日志在程序结束前被刷新。
    pub fn init(self) -> Option<WorkerGuard> {
        let level_filter = self
            .level
            .parse::<tracing_subscriber::filter::LevelFilter>()
            .unwrap_or(tracing_subscriber::filter::LevelFilter::INFO);
        let time_format = self.time_format.clone();

        // 1. 构建控制台层
        let console_layer = self.build_console_layer(&time_format, level_filter);

        // 2. 构建文件层
        let (file_layer, guard) = self.build_file_layer(&time_format, level_filter);

        // 3. 注册 Subscriber
        Registry::default()
            .with(console_layer)
            .with(file_layer)
            .init();

        // 4. 启动清理任务
        self.spawn_cleanup_task_if_needed();

        guard
    }

    /// 构建控制台输出层
    fn build_console_layer<S>(
        &self,
        time_format: &str,
        filter: tracing_subscriber::filter::LevelFilter,
    ) -> Option<impl Layer<S>>
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        if self.console {
            Some(
                fmt::layer()
                    .with_timer(ChronoLocal::new(time_format.to_string()))
                    .with_writer(std::io::stdout)
                    .with_filter(filter),
            )
        } else {
            None
        }
    }

    /// 构建文件输出层
    fn build_file_layer<S>(
        &self,
        time_format: &str,
        filter: tracing_subscriber::filter::LevelFilter,
    ) -> (Option<impl Layer<S>>, Option<WorkerGuard>)
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        if self.file {
            let file_appender = tracing_appender::rolling::daily(&self.log_dir, &self.file_prefix);
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            let layer = fmt::layer()
                .with_timer(ChronoLocal::new(time_format.to_string()))
                .with_ansi(false)
                .with_writer(non_blocking)
                .with_filter(filter);

            (Some(layer), Some(guard))
        } else {
            (None, None)
        }
    }

    /// 如果配置了清理策略，则启动后台清理任务
    fn spawn_cleanup_task_if_needed(&self) {
        if let Some(max_files) = self.max_files {
            if self.file {
                let log_dir = self.log_dir.clone();
                let file_prefix = self.file_prefix.clone();
                let interval = self.cleanup_interval;
                let max_files_usize = if max_files < 0 { 0 } else { max_files as usize };

                std::thread::spawn(move || {
                    loop {
                        // 执行清理
                        cleanup_old_logs(&log_dir, &file_prefix, max_files_usize);
                        // 等待下一次检查
                        std::thread::sleep(interval);
                    }
                });
            }
        }
    }
}

/// 执行清理逻辑：保留最新的 `max_files` 个日志文件
fn cleanup_old_logs(log_dir: &Path, file_prefix: &str, max_files: usize) {
    if !log_dir.exists() {
        return;
    }

    let read_dir = match std::fs::read_dir(log_dir) {
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to read log directory: {}", e);
            return;
        }
    };

    // 收集符合前缀的文件
    let mut log_files: Vec<_> = read_dir
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name().into_string().ok()?;

            if file_name.starts_with(file_prefix) {
                Some((entry, file_name))
            } else {
                None
            }
        })
        .collect();

    // 按文件名降序排序 (依赖于日期后缀格式为 ISO 8601 兼容，如 .2023-10-01)
    // 排序后：[app.log.2023-10-02, app.log.2023-10-01, ...]
    log_files.sort_by(|a, b| b.1.cmp(&a.1));

    // 删除多余的旧文件
    if log_files.len() > max_files {
        for (entry, _) in log_files.iter().skip(max_files) {
            if let Err(e) = std::fs::remove_file(entry.path()) {
                error!("Failed to remove old log file {:?}: {}", entry.path(), e);
            }
        }
    }
}
