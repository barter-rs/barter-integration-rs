use barter_integration::socket::{
    ExchangeSocket, Transformer,
    error::SocketError,
    protocol::websocket::{WebSocketParser, WebSocket, WsMessage}
};
use std::str::FromStr;
use serde::{de, Deserialize};
use serde_json::json;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;

// Convenient type alias for an `ExchangeSocket` utilising a tungstenite `WebSocket`
type ExchangeWebSocket<Exchange> = ExchangeSocket<WebSocketParser, WebSocket, Exchange, VolumeSum>;

// Communicative type alias for what the Transformer is generating
type VolumeSum = f64;

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum BinanceMessage {
    SubResponse {
        result: Option<Vec<String>>,
        id: u32,
    },
    Trade {
        #[serde(rename = "q", deserialize_with = "de_str")]
        quantity: f64,
    },
}

struct StatefulTransformer {
    sum_of_volume: VolumeSum,
}

impl Transformer<VolumeSum> for StatefulTransformer {
    type Input = BinanceMessage;
    type OutputIter = Vec<Result<VolumeSum, SocketError>>;

    fn transform(&mut self, input: Self::Input) -> Self::OutputIter {
        // Add new input Trade quantity to sum
        match input {
            BinanceMessage::SubResponse { .. } => {
                // Don't care about this for the example
            },
            BinanceMessage::Trade { quantity, .. } => {
                // Add new Trade volume to internal state VolumeSum
                self.sum_of_volume += quantity;
            },
        };

        // Return IntoIterator of length 1 containing the running sum of volume
        vec![Ok(self.sum_of_volume)]
    }
}

/// See Barter-Data for a comprehensive real-life example, as well as code you can use out of the
/// box to collect real-time public market data from many exchanges.
#[tokio::main]
async fn main() {
    // Establish Sink/Stream communication with desired WebSocket server
    let binance_conn = connect_async("wss://fstream.binance.com/ws/")
        .await
        .map(|(ws_conn, _)| ws_conn)
        .expect("failed to connect");

    // Instantiate some arbitrary Transformer to apply to data parsed from the WebSocket protocol
    let transformer = StatefulTransformer { sum_of_volume: 0.0 };

    // ExchangeWebSocket includes pre-defined WebSocket Sink/Stream & WebSocket ProtocolParser
    let mut websocket = ExchangeWebSocket::new(binance_conn, transformer);

    // Send something over the socket (eg/ Binance trades subscription)
    websocket
        .send(WsMessage::Text(
            json!({"method": "SUBSCRIBE","params": ["btcusdt@aggTrade"],"id": 1}).to_string()
        ))
        .await
        .expect("failed to send WsMessage over socket");

    // Receive a stream of your desired OutputData model from the socket
    while let Some(volume_result) = websocket.next().await {

        match volume_result {
            Ok(cumulative_volume) => {
                // Do something with your data
                println!("{cumulative_volume:?}");
            },
            Err(error) => {
                // React to any errors produced by the internal transformation
                eprintln!("{error}")
            }
        }
    }
}

/// Deserialize a string as the desired type
fn de_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: de::Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let data: String = Deserialize::deserialize(deserializer)?;
    data.parse::<T>().map_err(de::Error::custom)
}