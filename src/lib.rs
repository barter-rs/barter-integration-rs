#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms
)]
use crate::{
    protocol::StreamParser,
    error::SocketError
};
use std::{
    collections::VecDeque,
    fmt::Debug,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use futures::Stream;
use pin_project::pin_project;
use serde::Deserialize;

///! # Barter-Integration
///! Contains an [`ExchangeStream`] capable of acting as a [`Stream`] for a given remote server, and a [`ExchangeSink`]
///! capable of acting [`Sink`] for a given remote server.

/// Foundational data structures that define the building blocks used by the rest of the `Barter`
/// ecosystem.
///
/// eg/ `Market`, `Exchange`, `Instrument`, `Symbol`, etc.
pub mod model;

/// Custom `SocketError`s generated by an [`ExchangeStream`] and [`ExchangeSink`].
pub mod error;

/// Contains `StreamParser` implementations for transforming communication protocol specific
/// messages into a generic output data structure.
pub mod protocol;

/// Contains the flexible `Metric` type used for representing real-time metrics generically.
pub mod metric;

/// Utilities to assist deserialisation.
pub mod de;

/// [`Validator`]s are capable of determining if their internal state is satisfactory to fulfill
/// some use case defined by the implementor.
pub trait Validator {
    /// Check if `Self` is valid for some use case.
    fn validate(self) -> Result<Self, SocketError>
    where
        Self: Sized;
}

/// [`Transformer`]s are capable of transforming any `Input` into an iterator of
/// `Result<Output, SocketError>`s.
pub trait Transformer {
    type Input: for<'de> Deserialize<'de>;
    type Output;
    type OutputIter: IntoIterator<Item = Result<Self::Output, SocketError>>;
    fn transform(&mut self, input: Self::Input) -> Self::OutputIter;
}

/// An [`ExchangeStream`] is a communication protocol agnostic [`Stream`]. It polls protocol
/// messages from the inner [`Stream`], and transforms them into the desired output data structure.
#[derive(Debug)]
#[pin_project]
pub struct ExchangeStream<Protocol, InnerStream, StreamTransformer>
where
    Protocol: StreamParser,
    InnerStream: Stream,
    StreamTransformer: Transformer,
{
    #[pin]
    pub stream: InnerStream,
    pub transformer: StreamTransformer,
    pub buffer: VecDeque<Result<StreamTransformer::Output, SocketError>>,
    pub protocol_marker: PhantomData<Protocol>,
}

impl<Protocol, InnerStream, StreamTransformer> Stream
    for ExchangeStream<Protocol, InnerStream, StreamTransformer>
where
    Protocol: StreamParser,
    InnerStream: Stream<Item = Result<Protocol::Message, Protocol::Error>> + Unpin,
    StreamTransformer: Transformer,
{
    type Item = Result<StreamTransformer::Output, SocketError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Flush Self::Item buffer if it is not currently empty
            if let Some(output) = self.buffer.pop_front() {
                return Poll::Ready(Some(output));
            }

            // Poll inner `Stream` for next the next input protocol message
            let input = match self.as_mut().project().stream.poll_next(cx) {
                Poll::Ready(Some(input)) => input,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };

            // Parse input protocol message into `ExchangeMessage`
            let exchange_message = match Protocol::parse::<StreamTransformer::Input>(input) {
                // `StreamParser` successfully deserialised `ExchangeMessage`
                Some(Ok(exchange_message)) => exchange_message,

                // If `StreamParser` returns an Err pass it downstream
                Some(Err(err)) => return Poll::Ready(Some(Err(err))),

                // If `StreamParser` returns None it's a safe-to-skip message
                None => continue,
            };

            // Transform `ExchangeMessage` into `Transformer::OutputIter`
            // ie/ IntoIterator<Item = Result<Output, SocketError>>
            self.transformer
                .transform(exchange_message)
                .into_iter()
                .for_each(|output_result: Result<StreamTransformer::Output, SocketError>| {
                    self.buffer.push_back(output_result)
                });
        }
    }
}

impl<Protocol, InnerStream, StreamTransformer> ExchangeStream<Protocol, InnerStream, StreamTransformer>
where
    Protocol: StreamParser,
    InnerStream: Stream,
    StreamTransformer: Transformer,
{
    pub fn new(stream: InnerStream, transformer: StreamTransformer) -> Self {
        Self {
            stream,
            transformer,
            buffer: VecDeque::with_capacity(6),
            protocol_marker: PhantomData::default(),
        }
    }
}

// /// Todo:
// #[derive(Debug)]
// #[pin_project]
// pub struct ExchangeSink<Protocol, InnerSink, SinkTransformer, Output>
// where
//     Protocol: StreamParser,
//     // Todo: may not be Protocol::Message
//     InnerSink: Sink<Protocol::Message>,
//     // Todo: Transformer may need to be double generic or have a Transformer and a Sink/StreamTransformer that's associated
//     SinkTransformer: Transformer<ExchangeMessage>,
//     Output: Debug,
// {
//     #[pin]
//     pub sink: InnerSink,
//     pub transformer: SinkTransformer,
//     pub buffer: VecDeque<Result<Output, SocketError>>,
//     pub protocol_marker: PhantomData<Protocol>,
// }
//
// impl<Protocol, InnerSink, SinkTransformer, Output> Sink<Protocol::Message>
//     for ExchangeSink<Protocol, InnerSink, SinkTransformer, Output>
// where
//     Protocol: StreamParser,
//     InnerSink: Sink<Protocol::Message>,
//     SinkTransformer: Transformer<ExchangeMe>,
//     Output: Debug,
// {
//     type Error = SocketError;
//
//     fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         self.project()
//             .sink
//             .poll_ready(cx)
//             .map_err(|_| SocketError::Sink)
//     }
//
//     fn start_send(self: Pin<&mut Self>, item: Protocol::Message) -> Result<(), Self::Error> {
//         self.project()
//             .sink
//             .start_send(item)
//             .map_err(|_| SocketError::Sink)
//     }
//
//     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         self.project()
//             .sink
//             .poll_flush(cx)
//             .map_err(|_| SocketError::Sink)
//     }
//
//     fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         self.project()
//             .sink
//             .poll_close(cx)
//             .map_err(|_| SocketError::Sink)
//     }
// }
//
// impl<Protocol, InnerSink, SinkTransformer, Output>
//     ExchangeSink<Protocol, InnerSink, SinkTransformer, Output>
// where
//     Protocol: StreamParser,
//     InnerSink: Sink<Protocol::Message>,
//     SinkTransformer: Transformer,
//     Output: Debug,
// {
//     pub fn new(sink: InnerSink, transformer: SinkTransformer) -> Self {
//         Self {
//             sink,
//             transformer,
//             buffer: VecDeque::with_capacity(6),
//             protocol_marker: PhantomData::default(),
//         }
//     }
// }
