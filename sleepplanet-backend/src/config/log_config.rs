use serde::Deserialize;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;

use tracing_appender::rolling;

use super::default_true;

const FORMAT_PRETTY: &str = "pretty";
const FORMAT_COMPACT: &str = "compact";
const FORMAT_JSON: &str = "json";
const FORMAT_FULL: &str = "full";

#[derive(Deserialize, Clone, Debug)]
pub struct LogConfig {
    #[serde(default = "default_filter_level")]
    pub filter_level: String,
    #[serde(default = "default_true")]
    pub with_ansi: bool,
    #[serde(default = "default_true")]
    pub stdout: bool,
    #[serde(default = "default_directory")]
    pub directory: String,
    #[serde(default = "default_file_name")]
    pub file_name: String,
    #[serde(default = "default_rolling")]
    pub rolling: String,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_true")]
    pub with_level: bool,
    #[serde(default = "default_true")]
    pub with_target: bool,
    #[serde(default = "default_true")]
    pub with_thread_ids: bool,
    #[serde(default = "default_true")]
    pub with_thread_names: bool,
    #[serde(default = "default_true")]
    pub with_source_location: bool,
}
fn default_filter_level() -> String {
    "info".into()
}
fn default_directory() -> String {
    "./logs".into()
}
fn default_file_name() -> String {
    "app.log".into()
}
fn default_rolling() -> String {
    "daily".into()
}
fn default_format() -> String {
    FORMAT_FULL.into()
}

impl Default for LogConfig {
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
    pub fn filter_level(mut self, filter_level: &str) -> Self {
        self.filter_level = filter_level.to_owned();
        self
    }
    pub fn with_ansi(mut self, with_ansi: bool) -> Self {
        self.with_ansi = with_ansi;
        self
    }
    pub fn stdout(mut self, stdout: bool) -> Self {
        self.stdout = stdout;
        self
    }
    pub fn directory(mut self, directory: impl Into<String>) -> Self {
        self.directory = directory.into();
        self
    }
    pub fn file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = file_name.into();
        self
    }
    pub fn rolling(mut self, rolling: impl Into<String>) -> Self {
        let rolling = rolling.into();
        if !["minutely", "hourly", "daily", "never"].contains(&&*rolling) {
            panic!("rolling must be one of minutely, hourly, daily, never");
        }
        self.rolling = rolling;
        self
    }
    pub fn format(mut self, format: impl Into<String>) -> Self {
        let format = format.into();
        if format != FORMAT_PRETTY
            && format != FORMAT_COMPACT
            && format != FORMAT_JSON
            && format != FORMAT_FULL
        {
            panic!("format must be one of pretty, compact, json, full");
        }
        self.format = format;
        self
    }

    pub fn with_level(mut self, with_level: bool) -> Self {
        self.with_level = with_level;
        self
    }

    pub fn with_target(mut self, with_target: bool) -> Self {
        self.with_target = with_target;
        self
    }
    pub fn with_thread_ids(mut self, with_thread_ids: bool) -> Self {
        self.with_thread_ids = with_thread_ids;
        self
    }
    pub fn with_thread_names(mut self, with_thread_names: bool) -> Self {
        self.with_thread_names = with_thread_names;
        self
    }
    pub fn with_source_location(mut self, with_source_location: bool) -> Self {
        self.with_source_location = with_source_location;
        self
    }
    pub fn guard(self) -> WorkerGuard {
        // todo
// 根据 `self.rolling` 的值选择不同的日志滚动策略，创建一个文件追加器。
// `self.rolling` 可能的值有 "minutely", "hourly", "daily" 和其他值，其他值将使用 `never` 策略。
let file_appender = match &*self.rolling {
    // 每分钟滚动一次日志文件，将日志写入指定目录下的指定文件
    "minutely" => rolling::minutely(&self.directory, &self.file_name),
    // 每小时滚动一次日志文件，将日志写入指定目录下的指定文件
    "hourly" => rolling::hourly(&self.directory, &self.file_name),
    // 每天滚动一次日志文件，将日志写入指定目录下的指定文件
    "daily" => rolling::daily(&self.directory, &self.file_name),
    // 不进行日志文件滚动，所有日志都写入同一个文件
    _ => rolling::never(&self.directory, &self.file_name),
};
// 将文件追加器包装成非阻塞的写入器，同时获取一个 `WorkerGuard` 用于管理后台线程。
// `file_writer` 用于非阻塞地写入日志，`guard` 用于确保后台线程在合适的时候退出。
let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
// 创建一个 `tracing` 订阅器，用于格式化和过滤日志。
// 首先尝试从环境变量中获取日志过滤规则，如果获取失败，则使用 `self.filter_level` 作为过滤规则。
// 然后设置是否使用 ANSI 转义序列来美化日志输出。
let subscriber = tracing_subscriber::fmt().with_env_filter(
    // 尝试从默认环境变量中获取日志过滤规则
    tracing_subscriber::EnvFilter::try_from_default_env()
        // 如果获取失败，使用 `self.filter_level` 作为默认的日志过滤规则
        .unwrap_or(tracing_subscriber::EnvFilter::new(&self.filter_level)),
).with_ansi(self.with_ansi);


    }
}
