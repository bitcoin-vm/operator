
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::thread;
use std::time::Duration;
use nostr_sdk::prelude::*;

const MODNUM: u32 = u32::MAX;

fn main() {
    dynamic_ip(0);
}

// A BVM operator can be run on any instance with dynamic IP addresses.
// The operator regularly broadcasts their dynamic IP address from a dedicated Nostr account.
// Clients can retrieve the operator's IP address from the the operator's latest Nostr post.
// Nostr relays kind of act like DNS seeds

fn dynamic_ip(mut counter: u32) {
    let ip_result: Result<String> = return_ip();

    println!("counter: {}", counter);

    match ip_result {
        Ok(rst) => {
            println!("your dynamic ip address : {}", rst);

            try_post(&rst);
            counter = (counter + 1) % MODNUM;
        }
        Err(err) => {
            println!("ip err: {}", err);
            counter = (counter + 1) % MODNUM;
        }
    }

    thread::sleep(Duration::from_secs(10));
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
        thread::sleep(Duration::from_secs(10));
        try_post(ip_str)
    } 
}

#[tokio::main]
async fn post(ip_str: &str) -> Result<()> {

    // Dedicated npub: npub1s9qann6psxk7ycnafsxtcak9zy74flaflre03855n0zqn9uhhvusva958z
    let my_keys = Keys::parse("6406f0f588179677e3d782a434153bc6dab57c529be669626603b28b2d0d96dc")?;

    let client = Client::new(&my_keys);

    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

    client.add_relay("wss://relay.damus.io").await?;
    client
        .add_relay_with_opts(
            "wss://relay.nostr.info",
            RelayOptions::new().proxy(proxy).write(false),
        )
        .await?;
    client
        .add_relay_with_opts(
            "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
            RelayOptions::new().proxy(proxy),
        )
        .await?;

    client.connect().await;

    client
        .publish_text_note(ip_str.to_string(), [])
        .await?;

    Ok(())
}

