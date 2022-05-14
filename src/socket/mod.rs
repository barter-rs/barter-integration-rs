use crate::socket::{
    error::SocketError,
    protocol::ProtocolParser,
};
use std::{
    collections::VecDeque,
    task::{Context, Poll},
    pin::Pin,
    marker::PhantomData
};
use serde::de::DeserializeOwned;
use futures::{Sink, Stream};
use pin_project::pin_project;

pub mod protocol;
pub mod error;

/// `Transformer`s are capable of transforming any `Input` into an iterator of
/// `Result<Output, SocketError`s.
pub trait Transformer<Output> {
    type Input: DeserializeOwned;
    type OutputIter: IntoIterator<Item = Result<Output, SocketError>>;
    fn transform(&mut self, input: Self::Input) -> Self::OutputIter;
}

/// Todo:
#[derive(Debug)]
#[pin_project]
pub struct ExchangeSocket<Socket, Protocol, StreamTransformer, ExchangeMessage, Output>
where
    Protocol: ProtocolParser<ExchangeMessage>,
    Socket: Sink<Protocol::ProtocolMessage> + Stream,
    StreamTransformer: Transformer<Output>,
    ExchangeMessage: DeserializeOwned,
{
    #[pin]
    pub socket: Socket,
    pub transformer: StreamTransformer,
    pub buffer: VecDeque<Result<Output, SocketError>>,
    pub protocol_marker: PhantomData<Protocol>,
    pub exchange_message_marker: PhantomData<ExchangeMessage>,
}

impl<Socket, Protocol, StreamTransformer, ExchangeMessage, Output> Stream
    for ExchangeSocket<Socket, Protocol, StreamTransformer, ExchangeMessage, Output>
where
    Protocol: ProtocolParser<ExchangeMessage>,
    Socket: Sink<Protocol::ProtocolMessage> + Stream<Item = Protocol::ProtocolMessage> + Unpin,
    StreamTransformer: Transformer<Output, Input = ExchangeMessage>,
    ExchangeMessage: DeserializeOwned,
{
    type Item = Result<Output, SocketError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Flush Self::Item buffer if it is not currently empty
            if let Some(output) = self.buffer.pop_front() {
                return Poll::Ready(Some(output))
            }

            // Poll underlying `Stream` for next `StreamItem` input
            let input = match self.as_mut().project().socket.poll_next(cx) {
                Poll::Ready(Some(input)) => input,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };

            // Parse input `StreamItem` into `ExchangeMessage`
            let exchange_message = match Protocol::parse(input) {
                // `ProtocolParser` successfully deserialised `ExchangeMessage`
                Some(Ok(exchange_message)) => exchange_message,

                // If `ProtocolParser` returns an Err pass it downstream
                Some(Err(err)) => return Poll::Ready(Some(Err(err))),

                // If `ProtocolParser` returns None it's a safe-to-skip message
                None => return Poll::Pending,
            };

            // Transform `ExchangeMessage` into `Transformer::OutputIter`
            // ie/ IntoIterator<Item = Result<Output, SocketError>>
            self.transformer
                .transform(exchange_message)
                .into_iter()
                .for_each(|output| self.buffer.push_back(output));
        }
    }
}

impl<Socket, Protocol, StreamTransformer, ExchangeMessage, Output> Sink<Protocol::ProtocolMessage>
    for ExchangeSocket<Socket, Protocol, StreamTransformer, ExchangeMessage, Output>
where
    Protocol: ProtocolParser<ExchangeMessage>,
    Socket: Sink<Protocol::ProtocolMessage> + Stream,
    StreamTransformer: Transformer<Output>,
    ExchangeMessage: DeserializeOwned,
{
    type Error = SocketError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().socket.poll_ready(cx).map_err(|_| SocketError::Sink)
    }

    fn start_send(self: Pin<&mut Self>, item: Protocol::ProtocolMessage) -> Result<(), Self::Error> {
        self.project().socket.start_send(item).map_err(|_| SocketError::Sink)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().socket.poll_flush(cx).map_err(|_| SocketError::Sink)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().socket.poll_close(cx).map_err(|_| SocketError::Sink)
    }
}

impl<Socket, Protocol, StreamTransformer, ExchangeMessage, Output>
    ExchangeSocket<Socket, Protocol, StreamTransformer, ExchangeMessage, Output>
where
    Protocol: ProtocolParser<ExchangeMessage>,
    Socket: Sink<Protocol::ProtocolMessage> + Stream,
    StreamTransformer: Transformer<Output>,
    ExchangeMessage: DeserializeOwned,
{
    pub fn new(socket: Socket, transformer: StreamTransformer) -> Self {
        Self {
            socket,
            transformer,
            buffer: VecDeque::with_capacity(6),
            protocol_marker: PhantomData::default(),
            exchange_message_marker: PhantomData::default(),
        }
    }
}