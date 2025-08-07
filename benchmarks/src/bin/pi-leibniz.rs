#[no_mangle]
pub extern "C" fn calculate_pi(iter: i64) -> f64 {
    let mut pi: f64 = 0.0;
    let mut op: i32 = 1;

    for i in 0..iter {
        pi += op as f64 / (2 * i + 1) as f64;
        op *= -1;
    }

    4.0 * pi
}

fn main() {
    let result: f64 = calculate_pi(100000);
    println!("{}", result);
}
