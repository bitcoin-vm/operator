use nostr_sdk::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::thread;
use std::time::Duration;

const MODNUM: u32 = u32::MAX;

fn main() {
    let file = match File::open("src/ip.txt") {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("src/ip.txt") {
                Ok(file) => file,
                Err(_error) => panic!("could not create ip.txt"),
            },
            _ => panic!("unknown ip .txt error"),
        },
    };
    drop(file);

    println!("prev ip was: {}", retrieve_prev_ip());

    dynamic_ip(0);
}

fn retrieve_prev_ip() -> String {
    let mut file = File::open("src/ip.txt").unwrap();

    let mut ip_txt = String::new();
    file.read_to_string(&mut ip_txt).unwrap();

    ip_txt
}

// A BVM operator can be run on any instance with dynamic IP addresses.
// The operator regularly broadcasts their dynamic IP address from a dedicated Nostr account.
// Clients can retrieve the operator's IP address from the the operator's latest Nostr post.
// Nostr relays kind of act like DNS seeds

fn dynamic_ip(mut counter: u32) {
    println!("counter: {}", counter);
    let ip_result: Result<String> = return_ip();
    match ip_result {
        Ok(rst) => {
            if retrieve_prev_ip() != rst {
                println!("new ip detected: {}", rst);
                try_post(&rst);
                println!("new ip posted: {}", rst);

                let mut file = File::create("src/ip.txt").unwrap();
                file.write_all(rst.as_bytes()).unwrap();
            }
        }
        Err(_err) => println!("ip address could not be retrieved.")
    }
    counter = (counter + 1) % MODNUM;
    thread::sleep(Duration::from_secs(5));
    dynamic_ip(counter)
}

#[tokio::main]
async fn return_ip() -> Result<String> {
    let url = "https://api.ipify.org?format=json";

    let response = reqwest::get(url).await?;
    let json: serde_json::Value = response.json().await?;

    if let Some(ip) = json["ip"].as_str() {
        Ok(ip.to_string())
    } else {
        Err("Error message".into())
    }
}

fn try_post(ip_str: &str) {
    if let Err(_err) = post(ip_str) {
        thread::sleep(Duration::from_secs(1));
        try_post(ip_str)
    }
}

#[tokio::main]
async fn post(ip_str: &str) -> Result<String> {
    // Dedicated npub: npub15jmehrv33xzw5xgt2khp50daxgzvp7v0zrrwmcdzhvsjkz8h7lrsh8ygxq
    let my_keys = Keys::parse("0d30b892172b3b7af48c19224f577e8a94a49949ec65bdfbb8d18f455dd6d591")?;

    let client = Client::new(&my_keys);

    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

    client.add_relay("wss://relay.damus.io").await?;
    client
        .add_relay_with_opts(
            "wss://relay.nostr.info",
            RelayOptions::new().proxy(proxy).write(false),
        )
        .await.unwrap();

    client.connect().await;

    match client.publish_text_note(ip_str.to_string(), []).await {
        Ok(eventid) => Ok(eventid.to_string()),
        Err(_error) => Err("post error".into()),
    }
}
