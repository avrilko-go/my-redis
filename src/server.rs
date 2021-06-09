use tokio::net::{TcpListener, TcpStream};
use std::future::Future;
use crate::{db::Db};
use std::sync::Arc;
use tokio::sync::{Semaphore, broadcast, mpsc};
use tracing::{error, info, debug};
use std::net::SocketAddr;
use std::io::Error;
use tokio::time;
use crate::connection::Connection;
use crate::shutdown::Shutdown;
use crate::cmd::Command;

const MAX_CONNECT: usize = 250;

#[derive(Debug)]
struct Listener {
    db: Db,
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

impl Listener {
    async fn run(&mut self) -> crate::Result<()> {
        info!("初始化完，开始接收客户端请求");
        loop {
            // 限制最大连接数 (阻塞等待)
            self.limit_connections.acquire().await.unwrap().forget();
            let socket = self.accept().await?;
            let mut handler = Handler {
                db: self.db.clone(),
                connection: Connection::new(socket),
                limit_connections: self.limit_connections.clone(),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            // 开启一个task处理（非阻塞）
            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                }
            });
        }
    }

    async fn accept(&mut self) -> crate::Result<TcpStream> {
        // 初始化发生错误的时候重试等待时间
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }
            // 发生了错误但是没到64
            time::sleep(time::Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

struct Handler {
    db: Db,
    connection: Connection,
    limit_connections: Arc<Semaphore>,
    shutdown: Shutdown,
    // 这个是当Handler被drop时候，会把_shutdown_complete一起drop，触发shutdown_complete_rx（这里是自动触发的，不需要手动调用）
    _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {

        // 只有当连接没有结束时候才循环读取
        while !self.shutdown.is_shutdown() {
            // 读取Frame出来
            let maybe_frame = tokio::select! {
                res = self.connection.read_frame() => {
                    res?
                },
                _ = self.shutdown.recv() => {
                    return Ok(())
                }
            };
            let frame = match maybe_frame {
                Some(frame) => frame,
                None => return Ok(()),
            };


            // 处理Frame消息
            let cmd = Command::from_frame(frame)?;

            // 打印cmd并将错误传递到外层
            debug!(?cmd);

            // 处理每个连接
            cmd.apply(&self.db, &mut self.connection, &mut self.shutdown).await?;
        }

        Ok(())
    }
}


pub async fn run(listener: TcpListener, shutdown: impl Future) -> crate::Result<()> {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Listener {
        db: Db::new(),
        listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECT)),
        notify_shutdown,
        shutdown_complete_rx,
        shutdown_complete_tx,
    };

    // 监听信号退出程序
    tokio::select! {
        res = server.run() =>  {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        },
        _ = shutdown =>  {
            info!("shutdown info");
        }
    }

    // 运行到这表示需要释放资源了

    let Listener {
        notify_shutdown,
        mut shutdown_complete_rx,
        shutdown_complete_tx,
        ..
    } = server;

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    // 优雅的退出（等待所有任务处理完再退出）
    let _ = shutdown_complete_rx.recv().await;
    Ok(())
}

impl Drop for Handler {
    fn drop(&mut self) {
        // handler 触发drop代表一个连接已经结束，需要将信号量+1，这样listener那才能拿到信号量
        self.limit_connections.add_permits(1);
    }
}