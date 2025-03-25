use std::{
    env, fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
mod health_checker;
use health_checker::check_health;
use load_balancer::ThreadPool;

pub struct Message {
    pub message: String,
    pub worker_addr: String,
    pub client_addr: String,
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let worker: String = fs::read_to_string(".env").unwrap();
    let mut worker_list: Vec<String> = worker.split("\n").map(|x| x.to_string()).collect();
    worker_list.remove(worker_list.len() - 1);
    let num_workers = worker_list.len();
    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool: ThreadPool = ThreadPool::new(num_workers).unwrap();

    if args.len() > 1 {
        if args[1] == "-h" || args[1] == "--health-worker" {
            println!("Health mode on!");
            check_health(worker_list.clone());
        }
    }
    let mut worker: usize = 0;

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // think of a better way to handle this, cloning everything everytime is kinda bad
        let worker_id = worker.clone();
        let worker_list_cl = worker_list.clone();
        pool.execute(move || {
            handle_connection(stream, worker_id, worker_list_cl);
        });

        worker = (worker + 1) % num_workers;
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, worker_id: usize, worker_list: Vec<String>) {
    let buf_reader = BufReader::new(&stream);
    let request_message = buf_reader.lines().next().unwrap().unwrap();
    let client_addr = stream.peer_addr().unwrap().to_string();
    let worker_addr = worker_list[worker_id].clone();
    // client_worker_map.insert(client_addr, worker_addr);
    let message: Message = Message {
        message: request_message,
        worker_addr: worker_addr.clone(),
        client_addr: client_addr.clone(),
    };
    let response = send_job(worker_addr, message);
    stream.write_all(response.as_bytes()).unwrap();
}

fn send_job(client_addr: String, message: Message) -> String {
    println!("Trying to connect to worker at {}", &client_addr);
    let mut worker_connection = TcpStream::connect(&client_addr).unwrap();
    worker_connection
        .write_all(message.message.as_bytes())
        .unwrap();
    worker_connection.flush().unwrap();
    println!("Sent work to worker at {}", &client_addr);
    let mut response = String::new();
    let _ = worker_connection.read_to_string(&mut response);
    println!("Received response! {}", &response);
    response
}
