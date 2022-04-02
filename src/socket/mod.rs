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
use futures::{Sink, Stream};
use serde::de::DeserializeOwned;
use pin_project::pin_project;

pub mod protocol;
pub mod error;

/// Todo:
pub trait Transformer<Output> {
    type Input: DeserializeOwned;
    type OutputIter: IntoIterator<Item = Result<Output, SocketError>>;
    fn transform(&mut self, input: Self::Input) -> Self::OutputIter;
}

/// Todo:
#[derive(Debug)]
#[pin_project]
pub struct ExchangeSocket<Socket, SocketItem, StreamParser, StreamTransformer, ExchangeMessage, Output>
where
    Socket: Sink<SocketItem> + Stream,
    StreamParser: ProtocolParser<ExchangeMessage>,
    StreamTransformer: Transformer<Output>,
    ExchangeMessage: DeserializeOwned,
{
    #[pin]
    pub socket: Socket,
    pub parser: StreamParser,
    pub transformer: StreamTransformer,
    pub buffer: VecDeque<Result<Output, SocketError>>,
    pub socket_item_marker: PhantomData<SocketItem>,
    pub exchange_message_marker: PhantomData<ExchangeMessage>,
}

impl<Socket, SocketItem, StreamItem, StreamParser, StreamTransformer, ExchangeMessage, Output> Stream
    for ExchangeSocket<Socket, SocketItem, StreamParser, StreamTransformer, ExchangeMessage, Output>
where
    Socket: Sink<SocketItem> + Stream<Item = StreamItem> + Unpin,
    StreamParser: ProtocolParser<ExchangeMessage, Input = StreamItem>,
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
            let exchange_message = match StreamParser::parse(input) {
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

impl<Socket, SocketItem, StreamParser, StreamTransformer, ExchangeMessage, Output> Sink<SocketItem>
    for ExchangeSocket<Socket, SocketItem, StreamParser, StreamTransformer, ExchangeMessage, Output>
where
    Socket: Sink<SocketItem> + Stream,
    StreamParser: ProtocolParser<ExchangeMessage>,
    StreamTransformer: Transformer<Output>,
    ExchangeMessage: DeserializeOwned,
{
    type Error = SocketError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().socket.poll_ready(cx).map_err(|_| SocketError::SinkError)
    }

    fn start_send(self: Pin<&mut Self>, item: SocketItem) -> Result<(), Self::Error> {
        self.project().socket.start_send(item).map_err(|_| SocketError::SinkError)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().socket.poll_flush(cx).map_err(|_| SocketError::SinkError)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().socket.poll_close(cx).map_err(|_| SocketError::SinkError)
    }
}

impl<Socket, SocketItem, StreamParser, StreamTransformer, ExchangeMessage, Output>
    ExchangeSocket<Socket, SocketItem, StreamParser, StreamTransformer, ExchangeMessage, Output>
where
    Socket: Sink<SocketItem> + Stream,
    StreamParser: ProtocolParser<ExchangeMessage>,
    StreamTransformer: Transformer<Output>,
    ExchangeMessage: DeserializeOwned,
{
    pub fn new(socket: Socket, parser: StreamParser, transformer: StreamTransformer) -> Self {
        Self {
            socket,
            parser,
            transformer,
            buffer: VecDeque::with_capacity(6),
            socket_item_marker: PhantomData::default(),
            exchange_message_marker: PhantomData::default(),
        }
    }
}