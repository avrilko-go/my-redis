use structopt::StructOpt;
use my_redis::{Result, DEFAULT_PORT, server};
use tokio::net::{TcpListener};
use tokio::signal::ctrl_c;

#[tokio::main]
async fn main() -> Result<()> {
    // 开启日志
    tracing_subscriber::fmt::try_init()?;
    // 自定义传入的端口
    let cli = Cli::from_args();
    let port = cli.port.as_deref().unwrap_or(DEFAULT_PORT);
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;
    server::run(listener, ctrl_c()).await
}


#[derive(StructOpt, Debug)]
#[structopt(name = "my-redis-server", version = env ! ("CARGO_PKG_VERSION"), author = env ! ("CARGO_PKG_AUTHORS"), about = "A Redis server")]
struct Cli {
    #[structopt(name = "port", long = "--port")]
    port: Option<String>,
}