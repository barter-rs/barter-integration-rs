use barter_integration::socket::{
    Transformer,
    error::SocketError,
    protocol::websocket::{ExchangeWebSocket, WebSocketParser, WsMessage}
};
use futures::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use tokio_tungstenite::connect_async;
use serde::Deserialize;
use serde_json::json;
use std::ops::Add;

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum BinanceMessage {
    Trade {
        #[serde(rename = "q")]
        quantity: Decimal,
    },
}

struct StatefulTransformer {
    sum_of_volume: Decimal,
}

impl Transformer<Decimal> for StatefulTransformer {
    type Input = BinanceMessage;
    type OutputIter = Vec<Result<Decimal, SocketError>>;

    fn transform(&mut self, input: Self::Input) -> Self::OutputIter {
        // Add new input Trade quantity to sum
        match input {
            BinanceMessage::Trade { quantity, .. } => self.sum_of_volume.add(quantity)
        };

        // Return IntoIterator of length 1 containing the running sum of volume
        vec![Ok(self.sum_of_volume)]
    }
}

#[tokio::main]
async fn main() {
    // Establish Sink/Stream communication with desired WebSocket server
    let binance_conn = connect_async("wss://fstream.binance.com/ws/")
        .await
        .map(|(ws_conn, _)| ws_conn)
        .expect("failed to connect");

    // Instantiate some arbitrary Transformer to apply to data parsed from the WebSocket protocol
    let transformer = StatefulTransformer { sum_of_volume: Decimal::ZERO };

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
    while let Some(Ok(trade_price)) = websocket.next().await {

        // Do something with your data
        println!("{:?}", trade_price);
    }
}