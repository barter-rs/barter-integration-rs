pub mod model;

// /// [`Binance`] Message variants that could be received from the Binance `WebSocket` server.
// #[derive(Debug, Clone, Deserialize, Serialize)]
// #[serde(untagged)]
// pub enum BinanceMessage {
//     Trade(BinanceTrade),
// }
//
// /// [`Binance`] specific Trade message.
// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct BinanceTrade {
//     #[serde(rename = "e")]
//     event_type: String,
//     #[serde(rename = "E", skip_deserializing)]
//     event_time: i64,
//     #[serde(rename = "s")]
//     symbol: String,
//     #[serde(rename = "a")]
//     trade_id: u64,
//     #[serde(rename = "p", deserialize_with = "de_str_to_f64")]
//     price: f64,
//     #[serde(rename = "q", deserialize_with = "de_str_to_f64")]
//     quantity: f64,
//     #[serde(rename = "f", skip_deserializing)]
//     buyer_order_id: u64,
//     #[serde(rename = "l", skip_deserializing)]
//     seller_order_id: u64,
//     #[serde(rename = "T")]
//     trade_time: i64,
//     #[serde(rename = "m")]
//     buyer_is_market_maker: bool,
//     #[serde(rename = "M", skip_deserializing)]
//     deprecated: bool,
// }
//
// impl From<(Instrument, BinanceTrade)> for Trade {
//     fn from((instrument, binance_trade): (Instrument, BinanceTrade)) -> Self {
//         Self {
//             trade_id: binance_trade.trade_id.to_string(),
//             timestamp: DateTime::from_utc(
//                 NaiveDateTime::from_timestamp(binance_trade.trade_time / 1000, 0),
//                 Utc,
//             ),
//             instrument,
//             price: binance_trade.price,
//             quantity: binance_trade.quantity,
//             buyer: match binance_trade.buyer_is_market_maker {
//                 true => BuyerType::Maker,
//                 false => BuyerType::Taker,
//             },
//         }
//     }
// }