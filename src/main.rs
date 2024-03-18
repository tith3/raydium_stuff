use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use solana_sdk::transaction;
use solana_transaction_status::UiTransactionEncoding;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use std::str::FromStr;
use std::sync::Mutex;

const RAYDIUM_LIQUIDITY_POOL_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const WS_RPC_URL: &str = "wss://mainnet.helius-rpc.com/?api-key=";
const RPC_URL: &str = "https://api.mainnet-beta.solana.com/?api-key=";
const INSTRUCTION: &str = "initialize2";
#[tokio::main]
async fn main() {
    //get rpc url with key
    let rpc_url = Url::parse(format!("{}{}", RPC_URL, get_api_key()).as_str()).expect("Invalid RPC URL");
    //create rpc client
    let rpc_client = RpcClient::new(rpc_url);

    //get ws rpc url with key
    let ws_rpc_url = Url::parse(format!("{}{}", WS_RPC_URL, get_api_key()).as_str()).expect("Invalid WebSocket URL");
    let (ws_stream, _) = connect_async(ws_rpc_url).await.expect("Failed to connect");

    //split stream into sender and receiver
    let (mut write, read) = ws_stream.split();

    //Construct subscribe message
    let account_subscribe_message = Message::Text(
        r#"{
            "jsonrpc":"2.0",
            "id":420,
            "method":"accountSubscribe",
            "params":[
                "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
                {
                    "encoding":"jsonParsed",
                    "commitment":"confirmed"
                }
            ]
        }"#.to_string(),
    );

    let logs_subscribe_message = Message::Text(
        r#"{
            "jsonrpc":"2.0",
            "id":1,
            "method":"logsSubscribe",
            "params":[
                {
                    "mentions":["675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"]
                },
                {
                    "commitment":"confirmed"
                }
            ]
        }"#.to_string(),
    );

    let transaction_subscribe_message = Message::Text(
        r#"{
            "jsonrpc":"2.0",
            "id":1,
            "method":"logsSubscribe",
            "params":[
                {
                    "mentions":["675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"]
                },
                {
                    "commitment":"finalized"
                }
            ]
        }"#.to_string(),
    );

    let transaction_config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };
    //send subscribe message
    write.send(transaction_subscribe_message).await.expect("Failed to send subscribe message");

    let i = Mutex::new(0);
    //read messages
    read.for_each(|message| async {
        
        match message {
            Ok(message) => {
                //parse message
                let message = message.to_text().unwrap();
                //check if message containes instruction
                if message.contains(INSTRUCTION) {
                    //parse message
                    let message: serde_json::Value = serde_json::from_str(message).unwrap();
                    
                    //println!("{:?}", message[0]);
                    //get signature
                    let signature = match message["params"]["result"]["value"]["signature"].as_str() {
                        Some(signature) => signature,
                        None => {
                            println!("Error: Signature not found");
                            return;
                        }
                    };
                    //get transaction
                    let transaction = rpc_client.get_transaction_with_config(&Signature::from_str(signature).unwrap(), transaction_config);
                    match transaction {
                        Ok(transaction) => {
                            print!("{:?}", transaction);
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                 }

                
                //let message: serde_json::Value = serde_json::from_str(message).unwrap();
                // println!("{}", *i.lock().unwrap());
                // *i.lock().unwrap() += 1;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }).await;

}

fn get_api_key() -> String {
    //get key from first line of ignored/keys.txt
    let keys = std::fs::read_to_string("src/ignored/keys.txt").unwrap();
    let key = keys.lines().next().unwrap();
    key.to_string()
}

