//! sats_por_real
//! poast on x.com and nostr how many sats 1BRL buys
//! x.com/sats_por_real
//! sats_por_real@luisschwab.net

use dotenv::dotenv;
use nostr_sdk::prelude::*;
use serde_json::Value;
use std::env;
use thousands::Separable;
use tweety_rs::TweetyClient;

const POAST_X: bool = true;
const POAST_NOSTR: bool = true;

const API_COINGECKO: &str = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=brl";
const API_MEMPOOL: &str = "https://mempool.space/api/blocks/tip/height";

#[rustfmt::skip]
const NOSTR_RELAYS: &[&str] = &[
    "wss://nostr.luisschwab.net",
    "wss://relay.primal.net"
    // add your relays here
];

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let env = vec![
        "X_CONSUMER_KEY",
        "X_CONSUMER_SECRET",
        "X_ACCESS_TOKEN",
        "X_ACCESS_TOKEN_SECRET",
        "NOSTR_SEC",
    ];

    dotenv().ok();
    let mut kv = std::collections::HashMap::new();

    // save env to kv
    for var in &env {
        match env::var(var) {
            Ok(value) => {
                //println!("{}: {}", var, value);
                kv.insert(*var, value);
            }
            Err(e) => {
                println!("couldn't read {} from environment: {}", var, e);
                std::process::exit(-1);
            }
        }
    }

    // fetch price from coingecko
    let price_brl = match reqwest::get(API_COINGECKO)
        .await?
        .json::<Value>()
        .await?
        .get("bitcoin")
        .and_then(|btc| btc.get("brl"))
        .and_then(|price| price.as_f64())
    {
        Some(price) => price,
        None => {
            eprintln!("Failed to get BTC price");
            std::process::exit(1);
        }
    };

    // fetch current height from mempool.space
    let height = reqwest::get(API_MEMPOOL).await?.text().await?;

    let sats_por_real = 100_000_000 / price_brl as u64;

    let payload = format!("丰{} @ {}", sats_por_real, height.separate_with_commas());

    println!("{}", payload);

    // poast on x.com
    if POAST_X {
        let x_client = TweetyClient::new(
            &kv["X_CONSUMER_KEY"],
            &kv["X_ACCESS_TOKEN"],
            &kv["X_CONSUMER_SECRET"],
            &kv["X_ACCESS_TOKEN_SECRET"],
        );

        match x_client.post_tweet(&payload, None).await {
            Ok(_) => {
                println!("successfully poasted to x: https://x.com/sats_por_real");
            }
            Err(e) => {
                println!("error while poasting to https://x.com/sats_por_real: {:?}", e);
            }
        }
    }

    // poast on nostr
    if POAST_NOSTR {
        let key = match Keys::parse(&kv["NOSTR_SEC"]) {
            Ok(key) => key,
            Err(e) => {
                println!("error while parsing nostr private key: {:#?}", e);
                std::process::exit(-1);
            }
        };

        let nostr_client = Client::new(key.clone());
        for relay in NOSTR_RELAYS {
            nostr_client.add_relay(*relay).await.unwrap();
            nostr_client.connect().await;
        }

        let event = EventBuilder::text_note(payload)
            .sign_with_keys(&key)
            .expect("failed to create event");

        match nostr_client.send_event(event).await {
            Ok(output) => {
                println!("successfully poasted to nostr: https://njump.me/{}", output.id().to_hex());
            }
            Err(e) => {
                println!("error while poasting to nostr:\n{:#?}", e);
            }
        }
    }

    Ok(())
}
