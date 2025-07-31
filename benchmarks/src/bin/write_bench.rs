use std::env;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // デフォルト値
    let mut size = 1048576;
    let mut iterations = 100;
    let mut file = "bench_test_file.dat".to_string();

    // 簡単な引数解析
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--size" => {
                if i + 1 < args.len() {
                    size = args[i + 1].parse().unwrap_or(size);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--iterations" => {
                if i + 1 < args.len() {
                    iterations = args[i + 1].parse().unwrap_or(iterations);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--file" => {
                if i + 1 < args.len() {
                    file = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    // Prepare test data
    let test_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    // Write benchmark
    for i in 0..iterations {
        let filename = format!("{}_{}", file, i);
        let mut f = File::create(&filename)?;
        f.write_all(&test_data)?;
        f.flush()?;
    }

    println!("Write benchmark completed successfully");
    Ok(())
}
