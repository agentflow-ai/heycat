// Tests for the runtime module

use super::*;

#[test]
fn test_run_async_executes_simple_future() {
    let result = run_async(async { 42 });
    assert_eq!(result, 42);
}

#[test]
fn test_run_async_executes_async_operation() {
    let result = run_async(async {
        let a = 10;
        let b = 20;
        a + b
    });
    assert_eq!(result, 30);
}

#[test]
fn test_run_async_propagates_values() {
    let data = vec![1, 2, 3];
    let sum = run_async(async move {
        data.iter().sum::<i32>()
    });
    assert_eq!(sum, 6);
}

#[test]
fn test_run_async_with_tokio_sleep() {
    let start = std::time::Instant::now();
    run_async(async {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    });
    let elapsed = start.elapsed();
    assert!(elapsed >= std::time::Duration::from_millis(10));
}
