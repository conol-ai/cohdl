use super::*;
use std::net::Shutdown;

fn failed_event_write(viewers: &Arc<Viewers>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (conn, _) = listener.accept().unwrap();
    client
        .write_all(b"GET /api/events HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    conn.shutdown(Shutdown::Write).unwrap();
    assert!(handle(
        conn,
        &Mutex::new(State {
            model: "{}".into(),
            error: None
        }),
        &AtomicU64::new(1),
        Path::new(""),
        &[],
        Path::new(""),
        viewers
    )
    .is_err());
}

#[test]
fn multitab_failed_writes_release_only_owned_reservations_and_preserve_idle_authority() {
    let viewers = Arc::new(Viewers::default());
    assert!(!viewers.ever.load(Ordering::SeqCst));
    assert!(viewers.last_drop.lock().unwrap().is_none());
    failed_event_write(&viewers);
    assert_eq!(viewers.active.load(Ordering::SeqCst), 0);
    assert!(viewers.ever.load(Ordering::SeqCst));
    let before = viewers.last_drop.lock().unwrap().unwrap();
    let held: Vec<_> = (0..5).map(|_| ViewerGuard::new(&viewers)).collect();
    assert_eq!(viewers.active.load(Ordering::SeqCst), 5);
    failed_event_write(&viewers);
    assert_eq!(viewers.active.load(Ordering::SeqCst), 5);
    let after_overflow = *viewers.last_drop.lock().unwrap();
    assert_eq!(
        after_overflow,
        Some(before),
        "Rejected overflow must not reset idle time"
    );
    drop(held);
    assert_eq!(viewers.active.load(Ordering::SeqCst), 0);
    assert!(viewers.last_drop.lock().unwrap().unwrap() > before);
    let promoted = ViewerGuard::new(&viewers);
    assert_eq!(viewers.active.load(Ordering::SeqCst), 1);
    drop(promoted);
    assert_eq!(viewers.active.load(Ordering::SeqCst), 0);
}

#[test]
fn multitab_simultaneous_guard_drops_release_every_reservation_once() {
    use std::sync::Barrier;
    let viewers = Arc::new(Viewers::default());
    let held: Vec<_> = (0..5)
        .map(|_| ViewerGuard::new(&viewers).unwrap())
        .collect();
    assert!(ViewerGuard::new(&viewers).is_none());
    let barrier = Arc::new(Barrier::new(6));
    let workers: Vec<_> = held
        .into_iter()
        .map(|guard| {
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                drop(guard);
            })
        })
        .collect();
    barrier.wait();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(viewers.active.load(Ordering::SeqCst), 0);
    assert!(viewers.last_drop.lock().unwrap().is_some());
    let promoted: Vec<_> = (0..5)
        .map(|_| ViewerGuard::new(&viewers).unwrap())
        .collect();
    assert!(ViewerGuard::new(&viewers).is_none());
    assert_eq!(viewers.active.load(Ordering::SeqCst), 5);
    drop(promoted);
    assert_eq!(viewers.active.load(Ordering::SeqCst), 0);
}
