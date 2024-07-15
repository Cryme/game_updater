use std::collections::HashSet;
use strum::{Display, EnumIter};

#[derive(Debug)]
pub struct LogHolder {
    pub(crate) producers: HashSet<String>,
    pub(crate) max_level: LogLevel,

    pub(crate) logs: Vec<Log>,

    pub(crate) producer_filter: String,
    pub(crate) max_log_level: LogLevel,
    pub(crate) level_filter: LogLevelFilter,
}

impl LogHolder {
    pub const ALL: &'static str = "All";

    pub(crate) fn new() -> Self {
        let mut producers = HashSet::new();
        producers.insert(Self::ALL.to_string());

        Self {
            producers,
            max_level: LogLevel::Info,
            logs: vec![],
            producer_filter: LogHolder::ALL.to_string(),
            max_log_level: LogLevel::Info,
            level_filter: LogLevelFilter::Info,
        }
    }

    pub fn add(&mut self, log: Log) {
        self.producers.insert(log.producer.clone());
        self.max_level = log.level.max(self.max_level);
        self.logs.push(log);
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Display, EnumIter)]
#[repr(u8)]
pub enum LogLevelFilter {
    Debug,
    Info,
    Warning,
    Error,
    All = 255,
}

#[derive(Clone, Debug)]
pub struct Log {
    pub level: LogLevel,
    pub producer: String,
    pub log: String,
    pub time: i64,
}
