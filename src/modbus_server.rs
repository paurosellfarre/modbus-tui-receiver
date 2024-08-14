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
    input_registers: Arc<Mutex<HashMap<u16, i32>>>,
}

impl tokio_modbus::server::Service for ServerService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = Exception;
    type Future = future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let res = match req {
            Request::ReadInputRegisters(addr, cnt) => {
                //Generate random values for the requested registers
                let mut registers = self.input_registers.lock().unwrap();

                //print cnt
                println!("SERVER: ReadInputRegisters - addr: {addr}, cnt: {cnt}");

                let mut prefix = String::new();
                // Depending on the cnt, we will use a different prefix
                match cnt {
                    1 => {
                        prefix = "AI".to_string(); 
                    },
                    2 => {
                        prefix = "DI".to_string();
                    },
                    _ => {
                        println!("SERVER: Exception::IllegalDataValue - Unimplemented register count in request: {req:?}");
                    }
                }
                
                // Update the values with random data
                let response_values: Vec<u16> = (0..cnt)
                    .map(|i| {
                        let reg_addr = addr + i;
                        let value = generate_random_value(reg_addr, prefix.clone());
                        registers.insert(reg_addr, value);
                        value as u16
                    })
                    .collect();
                
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
        let input_registers: HashMap<u16, i32> = (0..20)
            .map(|i| (i, generate_random_value(i, "AI".to_string())))
            .collect();
        
        Self {
            input_registers: Arc::new(Mutex::new(input_registers)),
        }
    }
}

// Generate random values for the requested registers (FAKE DATA)
fn generate_random_value(reg_addr: u16, prefix: String) -> i32 {
    let mut rng = rand::thread_rng();
    

    // match prefix + reg_addr
    match prefix.as_str() {
        "AI" => {
            match reg_addr {
                10 => rng.gen_range(0..=15000), // AI10
                11 => rng.gen_range(-15000..=0), // AI11
                12 => rng.gen_range(0..=1_000_000), // AI12
                13 => rng.gen_range(-1_000_000..=0), // AI13
                17 => rng.gen_range(0..=12000) , // AI17
                18 => rng.gen_range(-15000..=15000), // AI18
                19 => rng.gen_range(0..=1000), // AI19
                20 => rng.gen_range(0..=1000), // AI20
                30 => rng.gen_range(0..=11000), // AI30
                50 => rng.gen_range(-15000..=15000), // AI50
                231 => rng.gen_range(0..=2047), // AI231
                232 => rng.gen_range(0..=2047), // AI232
                233 => rng.gen_range(0..=2047), // AI233
                _ => 0,
            }
        },
        "DI" => {
            match reg_addr {
                0 => rng.gen_range(0..=1), // DI0
                1 => rng.gen_range(0..=1), // DI1
                8 => rng.gen_range(0..=1), // DI8
                80 => rng.gen_range(0..=1), // DI80
                _ => 0,
            }
        },
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
