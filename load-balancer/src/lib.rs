use core::fmt;
use error::ThreadError;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{mpsc, Arc, Mutex},
    thread,
};
use tracing::{error, info};
pub mod error;
pub mod tests;

pub struct Message {
    pub message: String,
    pub worker_addr: String,
    pub client_addr: String,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "[MESSAGE]: message: {}\n\tworker_addr: {}\n\tclient_addr: {}\n\t",
            self.message, self.worker_addr, self.client_addr
        )
    }
}

/// ThreadPool:
/// Has two fields, workers and sender
/// Workers is a vector of workers (threads) initialized when the threadpool was created
/// Sender is the sender half of the mpsc channel created for communicating between worker threads and the
/// ThreadPool threads
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    hash_ring: BTreeMap<u64, usize>, // Hash ring mapping hash values to worker IDs
}

/// Job:
/// Job is just a type alias for the long type of the closure functions which is to be executed by
/// the workers
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new threadpool of 'size' number of threads
    /// # Errors
    /// Gives a ThreadError::InvalidSize error if the size is zero
    pub fn new(size: usize) -> Result<ThreadPool, ThreadError> {
        // assert!(size > 0);
        if size == 0 {
            return Err(ThreadError::InvalidSize(
                "Thread pool with size 0 cannot be initialized!".to_string(),
            ));
        }

        let (sender, receiver) = mpsc::channel();

        // Usage of Arc:
        // The receiver cannot be simply cloned and given to the newly created workers as it's
        // possible that they might encounter race conditions while sharing the same receiver.
        // Hence Rust itself does not allow this with its MPSC (Multiple-Producer-Single-Consumer)
        // channel architecture. Hence we use an Arc with a mutex in it so that all the threads can
        // have ownership to the receiver and will not meet race conditions thanks to the mutex
        // The mutex will ensure that only one thread at a time is looking to receive more jobs
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        let mut hash_ring = BTreeMap::new();

        for id in 0..size {
            // Multiple ownership being done here as the Arc gets bumped up for each worker
            workers.push(Worker::new(id, Arc::clone(&receiver)));

            // Add worker to the hash ring
            let hash = Self::hash_worker_id(id);
            hash_ring.insert(hash, id);
        }
        info!(name: "[LB THREADS INIT]", "{size} workers at your service!");
        Ok(ThreadPool {
            workers,
            sender: Some(sender),
            hash_ring,
        })
    }

    /// Hash a worker ID to place it on the hash ring
    fn hash_worker_id(id: usize) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(id.to_string());
        let result = hasher.finalize();
        let hash = u64::from_be_bytes(result[0..8].try_into().unwrap());
        info!(name: "[HASH WORKER]", "Worker ID: {}, Hash: {}", id, hash);
        hash
    }

    /// Find the nearest worker on the hash ring for a given client/request
    pub fn get_worker_for_request(&self, client_id: &str) -> usize {
        let mut hasher = Sha256::new();
        hasher.update(client_id);
        let result = hasher.finalize();
        let hash = u64::from_be_bytes(result[0..8].try_into().unwrap());
        info!(name: "[HASH CLIENT]", "Client ID: {}, Hash: {}", client_id, hash);

        // Find the nearest worker on the hash ring
        match self.hash_ring.range(hash..).next() {
            Some((_, &worker_id)) => {
                info!(name: "[HASH MATCH]", "Client Hash: {}, Assigned Worker ID: {}", hash, worker_id);
                worker_id
            }
            None => {
                let worker_id = *self.hash_ring.values().next().unwrap(); // Wrap around to the first worker
                info!(name: "[HASH WRAPAROUND]", "Client Hash: {}, Assigned Worker ID: {}", hash, worker_id);
                worker_id
            }
        }
    }

    /// Method for executing the given closure (anonymous function)
    /// The closure will be of type FnOnce() as it will only get exexecuted once (just like the
    /// definition of thread::spawn in std library) and it takes no arguments
    /// It will have Send trait to transfer the closure from one thread to another, and it will also
    /// have a static lifetime as we don't know how long a process will take for its completion in
    /// the thread
    pub fn execute<F>(&self, client_id: &str, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // New job instance for the worker threads
        let job = Box::new(f);

        // Find the appropriate worker for the client
        let worker_id = self.get_worker_for_request(client_id);
        info!(name: "[LB HASHING]", "Client {} assigned to worker {}", client_id, worker_id);

        // Unwrapping here in case the receiver does not receive the "job". This could happen in a
        // situation where all the threads stopped
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Taking the sender to invoke an Error in every worker which in turn will result in a
        // peaceful shutdown process as there is no infinite loop being executed by any worker
        // thread
        drop(self.sender.take());
        for worker in &mut self.workers {
            info!(name:"[LB THREAD DESTROY]","Shutting down worker {}", worker.id);

            // Takes ownership of the thread from the worker and leaves a None in its place in the
            // worker and waits for the thread to complete its job
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// Worker struct:
/// Has two fields, an id field which is just a number and a thread::JoinHandle to handle the
/// thread
// The option in the thread is for a smooth drop of the thread pool as the thread's ownership will
// get transferred to the main thread while joining.
// So while the worker is working normally, it will have Some(thread) and when it's about to
// shutdown, the moved thread will leave a None value
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

/// Worker::new function:
/// Takes in a usize number for the id and the receiver half of the mpsc channel created in
/// ThreadPool, creates a thread using a builder (which returns a Result<JoinHandle, Error>) with the receiver in its closure and then
/// returns the newly created Worker struct with the id and thread
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let handler = thread::Builder::new();
        let thread = handler
            .spawn(move || loop {
                // The first unwrap is when aquiring the lock fails, which can happen in case of a
                // poisoned mutex state when some other thread panicked and is still holding the
                // lock.
                // The second unwrap is for recv errors, like the Sender has shut down unexpectedly
                let message = receiver
                    .lock()
                    .map_err(|e| {
                        ThreadError::MutexError(
                            format!("Maybe a thread panicked somewhere? Can't lock the mutex:/\nHere's additional info: {e}")
                        )
                    })
                    .unwrap()
                    .recv();
                match message {
                    Ok(job) => {
                         info!(name: "[LB THREAD EXEC]", "Worker {id} got a job; Executing...");
                        job();
                    }
                    // Shutting down when the thread cannot find a sender
                    Err(_) => {
                        error!(name: "[LB THREAD ERROR]", "Cannot find sender\nWorker {id} disconnected; shutting down.");
                        break;
                    }
                }
            })
            .map_err(|e| {
                ThreadError::ThreadHandlerError(
                    format!("Whoops, a handler error.. Too many workers perhaps?\nHere's additional info: {e}"),
                )
            })
            .unwrap();
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
