use std::collections::HashMap;
use std::env;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let targets: Vec<String> = env::args().collect();
    dbg!(targets);
    for ua in utils::USER_AGENTS {
        println!("{}", ua);
    }
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("{resp:#?}");
    Ok(())
}