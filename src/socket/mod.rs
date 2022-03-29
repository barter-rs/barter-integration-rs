
pub mod protocol;
pub mod error;

use crate::socket::{
    error::SocketError,
    protocol::ProtocolParser
};
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use serde::de::DeserializeOwned;
use pin_project::pin_project;
use futures::{Sink, Stream};

pub trait Transformer<Output> {
    type Input: DeserializeOwned;
    type OutputIter: IntoIterator<Item = Output>;
    fn transform(&mut self, input: Self::Input) -> Result<Self::OutputIter, SocketError>;
}

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
    pub socket_item_marker: PhantomData<SocketItem>,
    pub exchange_message_marker: PhantomData<ExchangeMessage>,
    pub output_marker: PhantomData<Output>,
}

impl<Socket, SocketItem, StreamItem, StreamParser, StreamTransformer, ExchangeMessage, Output> Stream
    for ExchangeSocket<Socket, SocketItem, StreamParser, StreamTransformer, ExchangeMessage, Output>
where
    Socket: Sink<SocketItem> + Stream<Item = StreamItem>,
    StreamParser: ProtocolParser<ExchangeMessage, Input = StreamItem>,
    StreamTransformer: Transformer<Output, Input = ExchangeMessage>,
    ExchangeMessage: DeserializeOwned,
{
    type Item = Result<StreamTransformer::OutputIter, SocketError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.socket.poll_next(cx) {
            Poll::Ready(Some(input)) => {
                // Parse ExchangeMessage from Socket Stream<Item = StreamItem> & transform to Output
                match StreamParser::parse(input) {
                    // If parser returns None it's a safe-to-skip message
                    None => {
                        Poll::Pending
                    }
                    Some(Ok(exchange_message)) => {
                        // Transform: Result<Vec<MarketEvent>, SocketError>
                        // eg/ Err(Unidentifiable Message)
                        // eg/ Ok(None) for SubscriptionSuccess
                        // eg/ Ok(Some(MarketEvent).into_iter())

                        // alternative:
                        // Transform: Option<Result<Vec<MarketEvent>, SocketError>>

                        // alternative:
                        // Iterator<Result<MarketEvent, SocketError>>

                        //  '--> Wrapped in Option<TransformResult>
                        Poll::Ready(Some(this.transformer.transform(exchange_message)))
                    },
                    Some(Err(err)) => {
                        Poll::Ready(Some(Err(err)))
                    }
                }
            }
            Poll::Ready(None) => {
                Poll::Ready(None)
            }
            Poll::Pending => {
                Poll::Pending
            }
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
            socket_item_marker: PhantomData::default(),
            exchange_message_marker: PhantomData::default(),
            output_marker: PhantomData::default()
        }
    }
}