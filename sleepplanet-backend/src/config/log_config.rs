//! 日志配置模块
//! 基于 tracing 框架实现的日志系统配置，支持多种日志格式和滚动策略
//! 参考: https://github.com/clia/tracing-config/blob/main/src/lib.rs

use serde::Deserialize;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;

use tracing_appender::rolling;

use super::default_true;

/// 日志格式常量定义
const FORMAT_PRETTY: &str = "pretty";  // 美观格式，适合开发环境
const FORMAT_COMPACT: &str = "compact";  // 紧凑格式，节省空间
const FORMAT_JSON: &str = "json";      // JSON格式，适合日志系统解析
const FORMAT_FULL: &str = "full";      // 完整格式，包含所有可用信息

/// 日志配置结构体
/// 用于配置日志的输出方式、格式、级别等参数
#[derive(Deserialize, Clone, Debug)]
pub struct LogConfig {
    /// 日志过滤级别
    /// 例如: "info" 或 "mycrate=trace"
    #[serde(default = "default_filter_level")]
    pub filter_level: String,
    /// 是否启用ANSI颜色输出
    #[serde(default = "default_true")]
    pub with_ansi: bool,
    /// 是否输出到标准输出(stdout)
    #[serde(default = "default_true")]
    pub stdout: bool,
    /// 日志文件存储目录
    #[serde(default = "default_directory")]
    pub directory: String,
    /// 日志文件名
    #[serde(default = "default_file_name")]
    pub file_name: String,
    /// 日志滚动策略
    /// 有效值: minutely(每分钟) | hourly(每小时) | daily(每天) | never(不滚动)
    #[serde(default = "default_rolling")]
    pub rolling: String,
    /// 日志输出格式
    /// 有效值: pretty | compact | json | full
    #[serde(default = "default_format")]
    pub format: String,
    /// 是否在日志中包含级别信息
    #[serde(default = "default_true")]
    pub with_level: bool,
    /// 是否在日志中包含目标信息
    #[serde(default = "default_true")]
    pub with_target: bool,
    /// 是否在日志中包含线程ID
    #[serde(default = "default_true")]
    pub with_thread_ids: bool,
    /// 是否在日志中包含线程名称
    #[serde(default = "default_true")]
    pub with_thread_names: bool,
    /// 是否在日志中包含源代码位置
    #[serde(default = "default_true")]
    pub with_source_location: bool,
}

/// 默认日志过滤级别
fn default_filter_level() -> String {
    "info".into()
}

/// 默认日志文件目录
fn default_directory() -> String {
    "./logs".into()
}

/// 默认日志文件名
fn default_file_name() -> String {
    "app.log".into()
}

/// 默认日志滚动策略
fn default_rolling() -> String {
    "daily".into()
}

/// 默认日志输出格式
fn default_format() -> String {
    FORMAT_FULL.into()
}

impl Default for LogConfig {
    /// 创建默认的日志配置
    fn default() -> Self {
        Self {
            filter_level: default_filter_level(),
            with_ansi: true,
            stdout: false,
            directory: default_directory(),
            file_name: default_file_name(),
            rolling: default_rolling(),
            format: default_format(),
            with_level: true,
            with_target: true,
            with_thread_ids: true,
            with_thread_names: true,
            with_source_location: true,
        }
    }
}

#[allow(dead_code)]
impl LogConfig {
    /// 设置日志过滤级别
    /// 如果未设置，将尝试从环境变量获取，默认值为"info"
    /// 可以使用类似"info"或"mycrate=trace"的格式
    pub fn filter_level(mut self, filter_level: &str) -> Self {
        self.filter_level = filter_level.to_owned();
        self
    }

    /// 设置是否启用ANSI颜色符号
    pub fn with_ansi(mut self, with_ansi: bool) -> Self {
        self.with_ansi = with_ansi;
        self
    }

    /// 设置是否将日志输出到标准输出
    pub fn stdout(mut self, stdout: bool) -> Self {
        self.stdout = stdout;
        self
    }

    /// 设置日志文件存储目录
    pub fn directory(mut self, directory: impl Into<String>) -> Self {
        self.directory = directory.into();
        self
    }

    /// 设置日志文件名
    pub fn file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = file_name.into();
        self
    }

    /// 设置日志滚动策略
    /// 有效值: minutely(每分钟) | hourly(每小时) | daily(每天) | never(不滚动)
    /// 其他值将导致panic
    pub fn rolling(mut self, rolling: impl Into<String>) -> Self {
        let rolling = rolling.into();
        if !["minutely", "hourly", "daily", "never"].contains(&&*rolling) {
            panic!("未知的日志滚动策略: {}", rolling)
        }
        self.rolling = rolling;
        self
    }

    /// 设置日志输出格式
    /// 有效值: pretty | compact | json | full
    /// 其他值将导致panic
    pub fn format(mut self, format: impl Into<String>) -> Self {
        let format = format.into();
        if format != FORMAT_PRETTY
            && format != FORMAT_COMPACT
            && format != FORMAT_JSON
            && format != FORMAT_FULL
        {
            panic!("未知的日志格式: {}", format)
        }
        self.format = format;
        self
    }

    /// 设置是否在日志中包含级别信息
    pub fn with_level(mut self, with_level: bool) -> Self {
        self.with_level = with_level;
        self
    }

    /// 设置是否在日志中包含目标信息
    pub fn with_target(mut self, with_target: bool) -> Self {
        self.with_target = with_target;
        self
    }

    /// 设置是否在日志中包含线程ID
    pub fn with_thread_ids(mut self, with_thread_ids: bool) -> Self {
        self.with_thread_ids = with_thread_ids;
        self
    }

    /// 设置是否在日志中包含线程名称
    pub fn with_thread_names(mut self, with_thread_names: bool) -> Self {
        self.with_thread_names = with_thread_names;
        self
    }

    /// 设置是否在日志中包含源代码位置
    pub fn with_source_location(mut self, with_source_location: bool) -> Self {
        self.with_source_location = with_source_location;
        self
    }

    /// 初始化日志系统
    /// 返回一个WorkerGuard，调用者需要持有它以确保日志正确刷新
    pub fn guard(&self) -> WorkerGuard {
        // 初始化日志写入器
        let file_appender = match &*self.rolling {
            "minutely" => rolling::minutely(&self.directory, &self.file_name),
            "hourly" => rolling::hourly(&self.directory, &self.file_name),
            "daily" => rolling::daily(&self.directory, &self.file_name),
            "never" => rolling::never(&self.directory, &self.file_name),
            _ => rolling::never(&self.directory, &self.file_name),
        };
        let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

        // 初始化日志订阅器
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or(tracing_subscriber::EnvFilter::new(&self.filter_level)),
            )
            .with_ansi(self.with_ansi);

        // 根据不同格式配置订阅器
        if self.format == FORMAT_PRETTY {
            let subscriber = subscriber.event_format(
                fmt::format()
                    .pretty()
                    .with_level(self.with_level)
                    .with_target(self.with_target)
                    .with_thread_ids(self.with_thread_ids)
                    .with_thread_names(self.with_thread_names)
                    .with_source_location(self.with_source_location),
            );
            if self.stdout {
                subscriber.with_writer(std::io::stdout).init();
            } else {
                subscriber.with_writer(file_writer).init();
            };
        } else if self.format == FORMAT_COMPACT {
            let subscriber = subscriber.event_format(
                fmt::format()
                    .compact()
                    .with_level(self.with_level)
                    .with_target(self.with_target)
                    .with_thread_ids(self.with_thread_ids)
                    .with_thread_names(self.with_thread_names)
                    .with_source_location(self.with_source_location),
            );
            if self.stdout {
                subscriber.with_writer(std::io::stdout).init();
            } else {
                subscriber.with_writer(file_writer).init();
            };
        } else if self.format == FORMAT_JSON {
            let subscriber = subscriber.event_format(
                fmt::format()
                    .json()
                    .with_level(self.with_level)
                    .with_target(self.with_target)
                    .with_thread_ids(self.with_thread_ids)
                    .with_thread_names(self.with_thread_names)
                    .with_source_location(self.with_source_location),
            );
            if self.stdout {
                subscriber.json().with_writer(std::io::stdout).init();
            } else {
                subscriber.json().with_writer(file_writer).init();
            };
        } else if self.format == FORMAT_FULL {
            let subscriber = subscriber.event_format(
                fmt::format()
                    .with_level(self.with_level)
                    .with_target(self.with_target)
                    .with_thread_ids(self.with_thread_ids)
                    .with_thread_names(self.with_thread_names)
                    .with_source_location(self.with_source_location),
            );
            if self.stdout {
                subscriber.with_writer(std::io::stdout).init();
            } else {
                subscriber.with_writer(file_writer).init();
            };
        }

        // 返回guard，调用者需要持有它
        guard
    }
}