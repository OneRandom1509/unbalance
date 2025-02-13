#[cfg(test)]
mod tests {
    use crate::error::ThreadError;
    use crate::ThreadPool;

    #[test]
    fn zero_threads() {
        let pool = ThreadPool::new(0);
        assert!(pool.is_err());

        if let Err(ThreadError::InvalidSize(msg)) = pool {
            assert_eq!(msg, "Thread pool with size 0 cannot be initialized!");
        } else {
            panic!("Expected ThreadError::InvalidSize error, but got a different error type.");
        }
    }
}
