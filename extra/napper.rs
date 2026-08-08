use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    println!("GM!!");
    let start = Instant::now();
    let size = 42 * 1024;
    let mut data: Vec<u8> = vec![0; size];
    println!("Memory allocated!");
    sleep(Duration::from_millis(42));
    println!("Napped for {:?}", Instant::now().duration_since(start));
}
