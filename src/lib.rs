pub mod server;

pub mod db;

pub mod connection;

pub mod shutdown;

pub mod frame;

pub mod cmd;

// 默认端口
pub const DEFAULT_PORT: &str = "6379";


// 自定义Error
pub type Error = Box<dyn std::error::Error + Sync + Send>;

// 自定义Result
pub type Result<T> = std::result::Result<T, Error>;