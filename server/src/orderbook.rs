#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OberonProof {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SummaryReq {
    #[prost(message, optional, tag = "1")]
    pub proof: ::core::option::Option<OberonProof>,
    #[prost(string, tag = "2")]
    pub symbol: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Summary {
    /// encoded as a string to avoid rounding errors
    /// deserialize to a fixed point decimal number suitable for financial calculations
    #[prost(string, tag = "1")]
    pub spread: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub bids: ::prost::alloc::vec::Vec<Level>,
    #[prost(message, repeated, tag = "3")]
    pub asks: ::prost::alloc::vec::Vec<Level>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Level {
    #[prost(string, tag = "1")]
    pub exchange: ::prost::alloc::string::String,
    /// encoded as a string to avoid rounding errors
    /// deserialize to a fixed point decimal number suitable for financial calculations
    #[prost(string, tag = "2")]
    pub price: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub amount: ::prost::alloc::string::String,
}
/// Generated server implementations.
pub mod orderbook_aggregator_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with OrderbookAggregatorServer.
    #[async_trait]
    pub trait OrderbookAggregator: Send + Sync + 'static {
        /// Server streaming response type for the BookSummary method.
        type BookSummaryStream: futures_core::Stream<
                Item = Result<super::Summary, tonic::Status>,
            >
            + Send
            + 'static;
        async fn book_summary(
            &self,
            request: tonic::Request<super::SummaryReq>,
        ) -> Result<tonic::Response<Self::BookSummaryStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct OrderbookAggregatorServer<T: OrderbookAggregator> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: OrderbookAggregator> OrderbookAggregatorServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for OrderbookAggregatorServer<T>
    where
        T: OrderbookAggregator,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/orderbook.OrderbookAggregator/BookSummary" => {
                    #[allow(non_camel_case_types)]
                    struct BookSummarySvc<T: OrderbookAggregator>(pub Arc<T>);
                    impl<
                        T: OrderbookAggregator,
                    > tonic::server::ServerStreamingService<super::SummaryReq>
                    for BookSummarySvc<T> {
                        type Response = super::Summary;
                        type ResponseStream = T::BookSummaryStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SummaryReq>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).book_summary(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BookSummarySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: OrderbookAggregator> Clone for OrderbookAggregatorServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: OrderbookAggregator> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: OrderbookAggregator> tonic::server::NamedService
    for OrderbookAggregatorServer<T> {
        const NAME: &'static str = "orderbook.OrderbookAggregator";
    }
}
