use std::{
    collections::HashMap,
    future,
    net::SocketAddr,
    sync::{Arc, Mutex}
};

use tokio::net::TcpListener;

use tokio_modbus::{
    prelude::*,
    server::tcp::{accept_tcp_connection, Server},
};

use rand::Rng;

struct ServerService {
    input_registers: Arc<Mutex<HashMap<u16, i16>>>,
}

impl tokio_modbus::server::Service for ServerService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = Exception;
    type Future = future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let res = match req {
            Request::ReadInputRegisters(addr, cnt) => {
                
                // Update the values with random data
                let mut response_values: Vec<u16> = Vec::new();

                let value = generate_random_value(addr);

                match cnt {
                    // 32 bits registers = 2 16 bits registers
                    2 => {
                        // Split the value into two u16 registers
                        let high = (value >> 16) as u16;
                        let low = (value & 0xFFFF) as u16;
                        response_values.push(high);
                        response_values.push(low);
                    },
                    // registers that can be negative
                    3 => {
                        let negative_indicator = value < 0;
                        response_values.push(if negative_indicator { 1 } else { 0 }); // Negative indicator
                        response_values.push((value as u16) & 0xFFFF); // Value
                        response_values.push(value as u16);
                    },
                    _ => {
                        response_values.push(value as u16);
                    }
                }
                
                
                Ok(Response::ReadInputRegisters(response_values))
                
            }
            _ => {
                println!("SERVER: Exception::IllegalFunction - Unimplemented function code in request: {req:?}");
                Err(Exception::IllegalFunction)
            }
        };
        future::ready(res)
    }
}

impl ServerService {
    fn new() -> Self {
        let input_registers: HashMap<u16, i16> = (0..20).map(|i| (i, 0)).collect();
        
        Self {
            input_registers: Arc::new(Mutex::new(input_registers)),
        }
    }
}

// Generate random values for the requested registers (FAKE DATA)
fn generate_random_value(reg_addr: u16) -> i32 {
    let mut rng = rand::thread_rng();
 
    match reg_addr {
        10 => rng.gen_range(0..=15000), // AI10
        11 => rng.gen_range(-15000..=0), // AI11
        12 => rng.gen_range(0..=1_000_000), // AI12
        13 => rng.gen_range(0..=1_000_000), // AI13
        17 => rng.gen_range(0..=12000) , // AI17
        18 => rng.gen_range(-15000..=15000), // AI18
        19 => rng.gen_range(0..=1000), // AI19
        20 => rng.gen_range(0..=1000), // AI20
        30 => rng.gen_range(0..=11000), // AI30
        50 => rng.gen_range(0..=5000), // AI50
        231 => rng.gen_range(0..=2047), // AI231
        232 => rng.gen_range(0..=2047), // AI232
        233 => rng.gen_range(0..=2047), // AI233
        00 => rng.gen_range(0..=1), // DI0
        01 => rng.gen_range(0..=1), // DI1
        08 => rng.gen_range(0..=1), // DI8
        080 => rng.gen_range(0..=1), // DI80
        _ => 0,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_addr = "127.0.0.1:5502".parse().unwrap();

    server_context(socket_addr).await?;

    Ok(())
}

async fn server_context(socket_addr: SocketAddr) -> anyhow::Result<()> {
    println!("Starting up server on {socket_addr}");
    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);
    let new_service = |_socket_addr| Ok(Some(ServerService::new()));
    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, new_service)
    };
    let on_process_error = |err| {
        eprintln!("{err}");
    };
    server.serve(&on_connected, on_process_error).await?;
    Ok(())
}
