use crate::{
    Instrument,
    socket::{
        ExchangeSocket,
        error::SocketError,
        protocol::websocket::{WebSocket, WebSocketParser, WsMessage}
    },
    public::{
        StreamIdentifier,
        Exchange, Transformer,
        model::{Subscription, MarketEvent, Sequence, MarketData},
        binance::{StreamId, BinanceMessage},
    },
};
use std::collections::HashMap;
use std::ops::DerefMut;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::public::model::StreamMeta;

pub type BinanceFuturesStream = ExchangeSocket<WebSocket, WsMessage, WebSocketParser, BinanceFutures, BinanceMessage, MarketEvent>;
pub type BinanceFuturesItem = std::option::IntoIter<MarketEvent>;

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct BinanceFutures {
    pub streams: HashMap<StreamId, StreamMeta>
}

impl Exchange<BinanceMessage> for BinanceFutures {
    const EXCHANGE: &'static str = "binance_futures";
    const BASE_URL: &'static str = "wss://fstream.binance.com/ws";

    fn new() -> Self {
        Self { streams: HashMap::new() }
    }

    fn generate_subscriptions(&mut self, subscriptions: &[Subscription]) -> Vec<serde_json::Value> {
        // Map Barter Subscriptions to a vector of BinanceFutures specific channels
        let channels = subscriptions
            .into_iter()
            .map(|subscription| {
                // Determine the BinanceFutures specific channel for this Subscription
                let channel = BinanceFutures::get_stream_id(subscription);

                // Add channel with the associated original Subscription to the internal HashMap
                self.streams
                    .insert(channel.clone(), StreamMeta::new(subscription.clone()));

                channel
            })
            .collect::<Vec<StreamId>>();

        // Construct BinanceFutures specific subscription message for all desired channels
        vec![json!({
            "method": "SUBSCRIBE",
            "params": channels,
            "id": 1
        })]
    }
}

impl Transformer<BinanceMessage, MarketEvent> for BinanceFutures {
    type OutputIter = std::option::IntoIter<MarketEvent>; // Todo:

    fn transform(&mut self, input: BinanceMessage) -> Result<Self::OutputIter, SocketError> {
        match input {
            BinanceMessage::Subscribed(sub_confirm) => {
                if sub_confirm.result.is_some() {
                    Err(SocketError::SubscribeError(""))
                } else {
                    Ok(None.into_iter())
                }
            }
            BinanceMessage::Trade(trade) => {
                let (instrument, sequence) = self.get_stream_meta(
                    &trade.to_stream_id()
                )?;

                Ok(Some(MarketEvent::new(
                    sequence,
                    MarketData::from((BinanceFutures::EXCHANGE, instrument, trade))
                )).into_iter())
            }
        }
    }
}

impl BinanceFutures {
    fn get_stream_id(subscription: &Subscription) -> StreamId {
        match subscription {
            Subscription::Trades(instrument) => {
                StreamId(format!("{}{}@aggTrade", instrument.base, instrument.quote))
            }
        }
    }

    fn get_stream_meta(&mut self, stream_id: &StreamId) -> Result<(Instrument, Sequence), SocketError> {
        self.streams
            .get_mut(stream_id)
            .map(|stream_meta| {

                let instrument = match &stream_meta.subscription {
                    Subscription::Trades(instrument) => instrument.clone()
                };

                let sequence = stream_meta.sequence;
                *stream_meta.sequence.deref_mut() += 1;

                (instrument, sequence)
            })
            .ok_or_else(|| SocketError::Unidentifiable(stream_id.0.clone()))
    }
}