use std::env;
use std::io::Write;

use log::info;

use async_runtime::async_io::{Ipv4Addr, TcpListener, TcpStream};
use async_runtime::executor::{spawner_and_executor, Spawner};

fn init_log() {
    // format = [file:line] msg
    env::set_var("RUST_LOG", "info");
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}:{:>3}] {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args(),
            )
        })
        .init();
}

async fn handle_client(stream: TcpStream) {
    let mut buf = [0u8; 1024];
    info!("(handle client) {}", stream.raw_fd());
    loop {
        let n = stream.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        stream.write(&buf[..n]).await.unwrap();
    }
}

async fn server_loop(spawner: Spawner) {
    let addr = Ipv4Addr::new(127, 0, 0, 1);
    let port = 8080;
    let listener = TcpListener::bind(addr, port).unwrap();

    let incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = stream.unwrap();
        spawner.spawn(handle_client(stream));
    }
}

fn main() {
    init_log();

    let (spawner, mut executor) = spawner_and_executor();

    spawner.spawn(server_loop(spawner.clone()));

    executor.run();
}
