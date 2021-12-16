// #[warn(unused_variables)]
#![allow(unused)]

#[macro_use]
extern crate bincode;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate hex;
extern crate num_cpus;
#[macro_use]
extern crate log;
extern crate blake3;
extern crate env_logger;
extern crate threadpool;
extern crate tokio;
extern crate tokio_util;

mod amd;
mod config;
mod connection;
mod constant;
mod error;
mod frame;
mod gpu;
mod intel;
mod miner;
mod model;
mod nvidia;
mod pow;
mod serder;
mod worker;

use crate::frame::Frame;
use crate::model::Message;
use clap::{App, Arg, SubCommand};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let cpu = num_cpus::get();
    let num_cpu = format!("{}", cpu);
    let num = num_cpu.as_str();
    let matches = App::new("alephium miner")
        .version("1.0.0")
        .author("知命")
        .about("alephium miner server")
        .arg(
            Arg::with_name("ip")
                .short("i")
                .long("ip")
                .value_name("ip")
                .help("set connect alephium node miner ip")
                .default_value("127.0.0.1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("port")
                .help("set connect alephium node miner port")
                .default_value("10973")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("miner_type")
                .short("t")
                .long("type")
                .value_name("miner_type")
                .help("miner type: cpu、amd、nvidia")
                .default_value("cpu")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("worker")
                .short("w")
                .long("worker")
                .value_name("worker")
                .help("worker number")
                .default_value(num)
                .takes_value(true),
        )
        .get_matches();
    env_logger::init();
    info!("starting up");
    let ip = matches.value_of("ip").unwrap_or("127.0.0.1").to_string();
    let port = matches.value_of("port").unwrap_or("10973").to_string();
    let miner_type = matches.value_of("miner_type").unwrap_or("cpu").to_string();
    let worker_num = matches.value_of("worker").unwrap_or(num);
    let config = config::Config {
        ip,
        port,
        miner_type,
        worker_num: worker_num.parse::<usize>().unwrap_or(cpu),
    };

    info!("{:?}", config);
    let address = format!("{}:{}", config.ip, config.port);
    let option = bincode::config::Configuration::standard()
        .with_big_endian()
        .with_no_limit()
        .with_fixed_int_encoding();

    // more program logic goes here...
    let (tx, mut rx) = mpsc::channel::<model::Message>(2 * config.worker_num);
    let manager = tokio::spawn(async move {
        let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
        let mut conn = connection::Connection::new(client);
        loop {
            match conn.read_frame().await {
                Ok(val) => match val {
                    Some(val) => {
                        if let Frame::Bulk(bytes) = val {
                            let (msg, size) =
                                bincode::decode_from_slice::<Message, _>(bytes.as_ref(), option)
                                    .expect("decode_from_slice msg error");
                            //send worker
                        }
                    }
                    None => unreachable!(),
                },
                Err(err) => error!("read_frame error {}", err),
            }
            if let Some(val) = rx.recv().await {
                let data = bincode::encode_to_vec(val, option).expect("encode_to_vec msg error");
                //send server
                if let Err(err) = conn.write_frame(&Frame::Bulk(data.into())).await {
                    error!("write_frame error {}", err);
                }
            }
        }
    });
    manager.await;
}
