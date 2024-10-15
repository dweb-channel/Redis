use anyhow::Result;
use async_channel::{bounded, Receiver, Sender};
use bytes::{Bytes, BytesMut};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, signal, time::sleep};

/// worker 线程
async fn worker(
    worker_name: String,
    req_receiver: Arc<Receiver<(SocketAddr, Bytes)>>,
    res_sender: Arc<Sender<(SocketAddr, Bytes)>>,
) -> Result<()> {
    println!("{} running...", worker_name);
    loop {
        tokio::select! {
            res = req_receiver.recv() => {
                let (src, msg) = match res {
                    Ok(r) => r,
                    Err(e) => {
                        println!("{}, Unable to read data, error:{}", worker_name, e);
                        return Ok(());
                    }
                };

                // 处理逻辑, 等待1s...
                sleep(Duration::from_secs(1)).await;
                println!("{}, process the request, src:{:?}", worker_name, src);

                // 结果发送到回复队列
                match res_sender.send((src, msg)).await {
                    Ok(_) => {},
                    Err(e) => println!("{}, Failed to send message to reply queue, error:{}", worker_name, e),
                }
            }
            _ = signal::ctrl_c() => {
                println!("{}, exit...", worker_name);
                req_receiver.close();
                return Ok(());
            }
        }
    }
}

// server线程
async fn server(
    socket: UdpSocket,
    req_sender: Arc<Sender<(SocketAddr, Bytes)>>,
    res_receiver: Arc<Receiver<(SocketAddr, Bytes)>>,
) -> Result<()> {
    println!("server running...");
    loop {
        let mut buf = BytesMut::with_capacity(1024);
        buf.resize(1024, 0);

        tokio::select! {

            // socket中读取到数据, 发送到请求队列.
            res = socket.recv_from(&mut buf) => {
                let (len, src) = match res {
                    Ok(r) => r,
                    Err(e) => {
                        println!("Unable to read data, error:{}", e);
                        continue;
                    }
                };
                println!("receive socket data, send to request queue. src:{:?}...", src);
                buf.resize(len, 0);
                req_sender.send((src, buf.freeze())).await?;
            }

            // 接收回复队列的数据发送到socket客户端
            Ok((src, msg)) = res_receiver.recv() => {
                println!("receive response queue data, send to socket client:{:?}...", src);
                match socket.send_to(&msg, src).await {
                    Ok(_) => continue,
                    Err(e) => println!("Failed to send data to client({:?}), error:{:?}", src, e),
                }
            }

            _ = signal::ctrl_c() => {
                println!("exit...");
                res_receiver.close();
                return Ok(());
            }

        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:3053".parse::<SocketAddr>()?;
    let socket = UdpSocket::bind(addr).await?;

    // 请求队列
    let (req_sender, req_receiver) = bounded::<(SocketAddr, Bytes)>(1024);
    // 回复队列
    let (res_sender, res_receiver) = bounded::<(SocketAddr, Bytes)>(1024);

    let req_sender = Arc::new(req_sender);
    let req_receiver = Arc::new(req_receiver);

    let res_sender = Arc::new(res_sender);
    let res_receiver = Arc::new(res_receiver);

    for i in 0..4 {
        let req_receiver = req_receiver.clone();
        let res_sender = res_sender.clone();
        // worker 线程
        tokio::spawn(async move {
            let worker_name = format!("worker:<{:?}>", i);
            let _ = worker(worker_name, req_receiver, res_sender).await;
        });
    }

    // server 线程
    server(socket, req_sender, res_receiver).await
}
