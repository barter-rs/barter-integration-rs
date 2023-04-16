use crate::model::instrument::{kind::InstrumentKind, symbol::Symbol};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub mod kind;
pub mod symbol;

/// Barter representation of an `Instrument`. Used to uniquely identify a `base_quote` pair, and it's
/// associated instrument type.
///
/// eg/ Instrument { base: "btc", quote: "usdt", kind: Spot }
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Instrument {
    pub base: Symbol,
    pub quote: Symbol,
    #[serde(rename = "instrument_type")]
    pub kind: InstrumentKind,
}

impl Display for Instrument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}_{}, {})", self.base, self.quote, self.kind)
    }
}

impl<S> From<(S, S, InstrumentKind)> for Instrument
where
    S: Into<Symbol>,
{
    fn from((base, quote, kind): (S, S, InstrumentKind)) -> Self {
        Self {
            base: base.into(),
            quote: quote.into(),
            kind,
        }
    }
}

impl Instrument {
    /// Constructs a new [`Instrument`] using the provided configuration.
    pub fn new<S>(base: S, quote: S, kind: InstrumentKind) -> Self
    where
        S: Into<Symbol>,
    {
        Self {
            base: base.into(),
            quote: quote.into(),
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::instrument::kind::InstrumentKind;
    use serde::de::Error;

    #[test]
    fn test_de_instrument() {
        struct TestCase {
            input: &'static str,
            expected: Result<Instrument, serde_json::Error>,
        }

        let cases = vec![
            TestCase {
                // TC0: Valid Spot Instrument
                input: r##"{"base": "btc", "quote": "usd", "instrument_type": "spot" }"##,
                expected: Ok(Instrument::from(("btc", "usd", InstrumentKind::Spot))),
            },
            TestCase {
                // TC1: Valid FuturePerpetual Instrument
                input: r##"{"base": "btc", "quote": "usd", "instrument_type": "future_perpetual" }"##,
                expected: Ok(Instrument::from((
                    "btc",
                    "usd",
                    InstrumentKind::FuturePerpetual,
                ))),
            },
            TestCase {
                // TC2: Invalid Spot Instrument w/ numeric base
                input: r##"{ "base": 100, "quote": "usd", "instrument_type": "future_perpetual" }"##,
                expected: Err(serde_json::Error::custom("")),
            },
            TestCase {
                // TC3: Invalid Instrument w/ gibberish InstrumentKind
                input: r##"{"base": "btc", "quote": "usd", "instrument_type": "gibberish" }"##,
                expected: Err(serde_json::Error::custom("")),
            },
            TestCase {
                // TC4: Invalid Instrument w/ complete gibberish
                input: r##"{ "gibberish": "shouldfail"}"##,
                expected: Err(serde_json::Error::custom("")),
            },
        ];

        for (index, test) in cases.into_iter().enumerate() {
            let actual = serde_json::from_str::<Instrument>(test.input);

            match (actual, test.expected) {
                (Ok(actual), Ok(expected)) => {
                    assert_eq!(actual, expected, "TC{} failed", index)
                }
                (Err(_), Err(_)) => {
                    // Test passed
                }
                (actual, expected) => {
                    // Test failed
                    panic!("TC{index} failed because actual != expected. \nActual: {actual:?}\nExpected: {expected:?}\n");
                }
            }
        }
    }
}
