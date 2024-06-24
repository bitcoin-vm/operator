use reqwest::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::thread;
use std::time::Duration;
use nostr_sdk::prelude::*;

const MODNUM: u32 = u32::MAX;

fn main() {
    dynamic_ip(0);
}

// A BVM operator can run on any instance with dynamic IP addresses.
// The operator regularly broadcasts their dynamic IP address from a dedicated Nostr account.
// Clients can retrieve the operator's IP address from the latest Nostr post by the operator.
// Nostr relays kind of act like DNS seeds

fn dynamic_ip(mut counter: u32) {
    thread::sleep(Duration::from_secs(10));

    let ip_result: Result<String, Error> = return_ip();

    println!("counter: {}", counter);

    match ip_result {
        Ok(rst) => {
            println!("your dynamic ip address is: {}", rst);
            post_dynamic_ip(&rst);
            counter = (counter + 1) % MODNUM;
        }
        Err(err) => {
            println!("ip err, reason: {}", err);
            counter = (counter + 1) % MODNUM;
        }
    }

    dynamic_ip(counter)
}

#[tokio::main]
async fn return_ip() -> Result<String, Error> {
    let url = "https://api.ipify.org?format=json";

    let response = reqwest::get(url).await?;
    let json: serde_json::Value = response.json().await?;

    if let Some(ip) = json["ip"].as_str() {
        Ok(ip.to_string())
    } else {
        Ok(String::from(""))
    }
}

#[tokio::main]
async fn post_dynamic_ip(ip_str: &str) -> Result<()> {

    // Dedicated npub: npub1smzlmsztyk34m5pdcx5x6al0jzfxrwz3f6l9lpxrdnfteqth2p2qwchau8
    let my_keys = Keys::parse("6fe68964026142ea3b07d1c0c0d1b68192d54634c6d731d566907f8bf68e2efc")?;

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
