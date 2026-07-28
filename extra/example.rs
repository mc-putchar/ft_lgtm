//! Example program for testing the WASM compilation and WASI execution
//! Allocates a buffer with pseudo-random data and computes a checksum.

/// Simulate some work by allocating a buffer and filling it with pseudo-random data.
/// The function takes the size of the buffer in megabytes and returns the checksum of the data.
fn do_work(size_mb: usize) {
    let size = size_mb * 1024 * 1024;
    let mut data: Vec<u8> = vec![0; size];

    let mut state: u64 = 0x4242_DEAD_BEEF_CAFE;
    for i in 0..size {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        data[i] = (state & 0xFF) as u8;
    }

    let checksum: u64 = data.iter().fold(0u64, |acc, &x| acc.wrapping_add(x as u64));

    println!("Processed {} bytes. Checksum: {}", data.len(), checksum);
}

fn main() {
    println!("Work size: 1 MB");
    do_work(1 as usize);
    println!("Work size: 2 MB");
    do_work(2 as usize);
    println!("Work size: 3 MB");
    do_work(3 as usize);
    println!("Work size: 4 MB");
    do_work(4 as usize);
    println!("Work size: 5 MB");
    do_work(5 as usize);

    // println!("Work size: 6 MB");
    // do_work(6 as usize);
    // println!("Work size: 7 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);

    // These will fail with OOM
    // println!("Work size: 9 MB");
    // do_work(9 as usize);
    // println!("Work size: 10 MB");
    // do_work(10 as usize);

    // The rest is just to waste more fuel

    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);

    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);

    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);

    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);

    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
    // println!("Work size: 8 MB");
    // do_work(7 as usize);
}
