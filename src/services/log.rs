#[derive(Debug)]
pub struct Log {
    pub msg: String,
}

pub fn log(log: Log) {
    println!("LOG: {:?}", log);
}
