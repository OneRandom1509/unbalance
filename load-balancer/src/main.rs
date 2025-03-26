mod health_checker;
use health_checker::check_health;
use load_balancer::{Message, ThreadPool};
use std::{
    env, fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info};
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();

    let addr_file = ".env";
    let args: Vec<String> = env::args().collect();

    let worker: String = fs::read_to_string(addr_file).unwrap();
    let mut worker_list: Vec<String> = worker.split("\n").map(|x| x.to_string()).collect();
    worker_list.remove(worker_list.len() - 1); // Removing the empty string
    let num_workers = worker_list.len();
    info!(name: "[FILE]", "Read {} addresses from {} file", &num_workers, &addr_file);

    let listener: TcpListener = TcpListener::bind("0.0.0.0:7878").unwrap();
    info!(name: "[CLIENT LISTENER]", "Load balancer listening on port 7878!");

    let pool: ThreadPool = ThreadPool::new(num_workers).unwrap();
    info!(name: "[LB THREAD POOL]", "Created a thread pool of {} thread(s)", &num_workers);

    if args.len() > 1 {
        if args[1] == "-h" || args[1] == "--health-checker" {
            info!(name:"[HEALTH CHECK]", "Health checker mode on!");
            check_health(worker_list.clone());
        } else {
            info!(name: "[UNRECOGNIZED ARG]", "Unrecognized argument");
        }
    } else {
        info!(name: "[NO ARGS]", "No command line args given, resorting to default");
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
    info!(name: "[LB SHUTDOWN]", "Shutting down.");
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
    debug!(name: "[MESSAGE]", "{}", &message);

    let response = forward_request(worker_addr, message);
    stream.write_all(response.as_bytes()).unwrap();
}

fn forward_request(client_addr: String, message: Message) -> String {
    info!(name: "[WORKER CONNECTION]", "Trying to connect to worker at {}", &client_addr);
    let mut worker_connection = TcpStream::connect(&client_addr).unwrap();
    info!(name: "[WORKER CONNECTION]", "Connected to worker at {}", &client_addr);

    worker_connection
        .write_all(message.message.as_bytes())
        .unwrap();
    worker_connection.flush().unwrap();
    info!(name: "[FORWARD REQUEST]", "Sent work to worker at {}", &client_addr);

    let mut response = String::new();
    let bytes_read = worker_connection.read_to_string(&mut response).unwrap();
    info!(name: "[RESPONSE RECEIVED]", "Received response of length {}!", &bytes_read);

    response
}
