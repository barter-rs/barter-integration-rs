use crate::{
    Instrument,
    socket::error::SocketError,
    public::{
        Exchange, Transformer,
        model::{Subscription, MarketData, Trade, Direction},
        binance::{BinanceStreamId, BinanceMessage, BinanceTrade},
    },
    util::epoch_ms_to_datetime_utc,
};
use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct BinanceFutures {
    pub subscriptions: HashMap<BinanceStreamId, Subscription>
}

impl Exchange<BinanceMessage> for BinanceFutures {
    const EXCHANGE: &'static str = "binance_futures";
    const BASE_URL: &'static str = "wss://fstream.binance.com/ws";

    fn new() -> Self {
        Self { subscriptions: HashMap::new() }
    }

    fn generate_subscriptions(&mut self, subscriptions: &[Subscription]) -> Vec<serde_json::Value> {
        // Map Barter Subscriptions to a vector of BinanceFutures specific channels
        let channels = subscriptions
            .into_iter()
            .map(|subscription| {
                // Determine the BinanceFutures specific channel for this Subscription
                let channel = BinanceFutures::get_stream_id(subscription);

                // Add channel with the associated original Subscription to the internal HashMap
                self.subscriptions.insert(channel.clone(), subscription.clone());

                channel
            })
            .collect::<Vec<String>>();

        // Construct BinanceFutures specific subscription message for all desired channels
        vec![json!({
            "method": "SUBSCRIBE",
            "params": channels,
            "id": 1
        })]
    }
}

impl Transformer<BinanceMessage, MarketData> for BinanceFutures {
    type OutputIter = std::option::IntoIter<MarketData>; // Todo:

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
                Ok(Some(MarketData::Trade(Trade {
                    id: trade.id.to_string(),
                    exchange: BinanceFutures::EXCHANGE.to_string(),
                    instrument: self.get_instrument(&trade)?,
                    received_timestamp: Utc::now(),
                    exchange_timestamp: epoch_ms_to_datetime_utc(trade.trade_ts),
                    price: trade.price,
                    quantity: trade.quantity,
                    direction: if trade.buyer_is_maker { // Todo: Check this
                        Direction::Short
                    } else {
                        Direction::Long
                    }
                })).into_iter())
            }
        }
    }
}

impl BinanceFutures {
    fn get_stream_id(subscription: &Subscription) -> BinanceStreamId {
        match subscription {
            Subscription::Trades(instrument) => {
                format!("{}{}@aggTrade", instrument.base, instrument.quote)
            }
        }
    }

    fn get_instrument(&self, trade: &BinanceTrade) -> Result<Instrument, SocketError> {
        self.subscriptions
            .get(&format!("{}@{}", trade.symbol.to_lowercase(), trade.event_type))
            .map(|subscription| match subscription {
                Subscription::Trades(instrument) => instrument.clone()
            })
            .ok_or(SocketError::Unidentifiable(format!("{:?}", trade)))
    }
}