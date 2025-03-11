fn main() {
    println!("Hello, world!");
    println!("global: {}", std::env::var("GLOBAL").unwrap());
    println!("local: {}", std::env::var("LOCAL").unwrap());
}
