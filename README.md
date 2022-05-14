# Barter-Integration

High-performance, low-level framework for composing flexible web integrations. The core Socket abstraction
provides customisable communication over any asynchronous protocol (WebSocket, FIX, etc.), with built-in translation
between server & client data models.

Utilised by other [`Barter`] trading ecosystem crates to build robust financial exchange integrations,
primarily for public data collection & trade execution. It is:
* **Low-Level**: Translates raw data streams communicated over the web into any desired data model using arbitrary data transformations.
* **Flexible**: Compatible with any protocol (WebSocket, FIX, etc.), any input/output model, and any user defined transformations. 

**See: [`Barter`], [`Barter-Data`] & [`Barter-Execution`]**

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

[crates-badge]: https://img.shields.io/crates/v/barter-integration.svg
[crates-url]: https://crates.io/crates/barter-integration

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://gitlab.com/open-source-keir/financial-modelling/trading/barter-integration-rs/-/blob/main/LICENCE

[actions-badge]: https://gitlab.com/open-source-keir/financial-modelling/trading/barter-integration-rs/badges/-/blob/main/pipeline.svg
[actions-url]: https://gitlab.com/open-source-keir/financial-modelling/trading/barter-integration-rs/-/commits/main

[discord-badge]: https://img.shields.io/discord/910237311332151317.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/wE7RqhnQMV

[API Documentation] | [Chat]

[`Barter`]: https://crates.io/crates/barter
[`Barter-Data`]: https://crates.io/crates/barter-data
[`Barter-Execution`]: https://crates.io/crates/barter-execution
[API Documentation]: https://docs.rs/barter-data/latest/barter_integration
[Chat]: https://discord.gg/wE7RqhnQMV

## Overview
Barter-Integration is a high-performance, low-level framework for composing flexible web integrations. It presents an 
extensible core abstraction called the ExchangeSocket. At a high level, an ExchangeSocket is made up of a few major 
components:
* Inner Socket that implements the Stream & Sink trait. 
* ProtocolParser trait implementation that is capable of parsing the input messages from a given protocol 
  (eg/ WebSocket, FIX, etc.) and deserialising into some output.
* Transformer trait implementation that transforms any input into an iterator of outputs.  

## Example

Binance tick-by-tick Trade consumer with Barter-Data.

```rust,no_run
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
```
**For a larger, "real world" example, see the [`Barter-Data`] repository.**

## Getting Help
Firstly, see if the answer to your question can be found in the [API Documentation]. If the answer is not there, I'd be
happy to help to [Chat] and try answer your question via Discord.

## Contributing
Thanks for your help in improving the Barter ecosystem! Please do get in touch on the discord to discuss
development, new features, and the future roadmap.

## Related Projects
In addition to the Barter-Integration crate, the Barter project also maintains:
* [`Barter`]: High-performance, extensible & modular trading components with batteries-included. Contains a
  pre-built trading Engine that can serve as a live-trading or backtesting system.
* [`Barter-Data`]: A high-performance WebSocket integration library for streaming public data from leading 
  cryptocurrency exchanges.
* [`Barter-Execution`]: Financial exchange integrations for trade execution - yet to be released!

## Roadmap
* Add new default ProtocolParser implementations to enable integration with other popular systems such as Kafka. 

## Licence
This project is licensed under the [MIT license].

[MIT license]: https://gitlab.com/open-source-keir/financial-modelling/trading/barter-data-rs/-/blob/main/LICENSE

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Barter-Integration by you, shall be licensed as MIT, without any additional
terms or conditions.