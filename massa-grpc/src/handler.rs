//! Copyright (c) 2022 MASSA LABS <info@massa.net>
//! gRPC API for a massa-node

use massa_proto::massa::api::v1::{
    self as grpc, GetBlocksBySlotRequest, GetBlocksBySlotResponse, GetNextBlockBestParentsRequest,
    GetNextBlockBestParentsResponse, GetTransactionsThroughputRequest,
    GetTransactionsThroughputResponse, GetTransactionsThroughputStreamRequest,
    SubscribeNewOperationsStreamRequest,
};

use crate::api::{
    get_blocks_by_slots, get_datastore_entries, get_next_block_best_parents, get_selector_draws,
    get_transactions_throughput, get_version,
};
use crate::service::MassaGrpcService;
use crate::stream::subscribe_new_operations::{
    subscribe_new_operations, SubscribeNewOperationsStream,
};
use crate::stream::subscribe_tx_throughput::{
    subscribe_transactions_throughput, SubscribeTransactionsThroughputStream,
};
use crate::stream::{
    send_blocks::{send_blocks, SendBlocksStream},
    send_endorsements::{send_endorsements, SendEndorsementsStream},
    send_operations::{send_operations, SendOperationsStream},
};
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl grpc::grpc_server::Grpc for MassaGrpcService {
    /// Handler for get multiple datastore entries.
    async fn get_datastore_entries(
        &self,
        request: tonic::Request<grpc::GetDatastoreEntriesRequest>,
    ) -> Result<tonic::Response<grpc::GetDatastoreEntriesResponse>, tonic::Status> {
        match get_datastore_entries(self, request) {
            Ok(response) => Ok(tonic::Response::new(response)),
            Err(e) => Err(e.into()),
        }
    }

    /// Handler for get version
    async fn get_version(
        &self,
        request: tonic::Request<grpc::GetVersionRequest>,
    ) -> Result<tonic::Response<grpc::GetVersionResponse>, tonic::Status> {
        match get_version(self, request) {
            Ok(response) => Ok(tonic::Response::new(response)),
            Err(e) => Err(e.into()),
        }
    }

    /// Handler for get selector draws
    async fn get_selector_draws(
        &self,
        request: Request<grpc::GetSelectorDrawsRequest>,
    ) -> Result<Response<grpc::GetSelectorDrawsResponse>, tonic::Status> {
        match get_selector_draws(self, request) {
            Ok(response) => Ok(tonic::Response::new(response)),
            Err(e) => Err(e.into()),
        }
    }

    /// Handler for get_next_block_best_parents
    async fn get_next_block_best_parents(
        &self,
        request: Request<GetNextBlockBestParentsRequest>,
    ) -> Result<Response<GetNextBlockBestParentsResponse>, Status> {
        match get_next_block_best_parents(self, request) {
            Ok(response) => Ok(Response::new(response)),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_transactions_throughput(
        &self,
        request: Request<GetTransactionsThroughputRequest>,
    ) -> Result<Response<GetTransactionsThroughputResponse>, Status> {
        match get_transactions_throughput(self, request) {
            Ok(response) => Ok(Response::new(response)),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_blocks_by_slot(
        &self,
        request: Request<GetBlocksBySlotRequest>,
    ) -> Result<Response<GetBlocksBySlotResponse>, Status> {
        match get_blocks_by_slots(self, request) {
            Ok(response) => Ok(Response::new(response)),
            Err(e) => Err(e.into()),
        }
    }

    // ███████╗████████╗██████╗ ███████╗ █████╗ ███╗   ███╗
    // ██╔════╝╚══██╔══╝██╔══██╗██╔════╝██╔══██╗████╗ ████║
    // ███████╗   ██║   ██████╔╝█████╗  ███████║██╔████╔██║
    // ╚════██║   ██║   ██╔══██╗██╔══╝  ██╔══██║██║╚██╔╝██║
    // ███████║   ██║   ██║  ██║███████╗██║  ██║██║ ╚═╝ ██║

    type SendBlocksStream = SendBlocksStream;
    type SendEndorsementsStream = SendEndorsementsStream;
    type SendOperationsStream = SendOperationsStream;
    type SubscribeTransactionsThroughputStream = SubscribeTransactionsThroughputStream;
    type SubscribeNewOperationsStream = SubscribeNewOperationsStream;

    /// Handler for send_blocks_stream
    async fn send_blocks(
        &self,
        request: tonic::Request<tonic::Streaming<grpc::SendBlocksRequest>>,
    ) -> Result<tonic::Response<Self::SendBlocksStream>, tonic::Status> {
        match send_blocks(self, request).await {
            Ok(res) => Ok(tonic::Response::new(res)),
            Err(e) => Err(e.into()),
        }
    }

    /// Handler for send_endorsements
    async fn send_endorsements(
        &self,
        request: tonic::Request<tonic::Streaming<grpc::SendEndorsementsRequest>>,
    ) -> Result<tonic::Response<Self::SendEndorsementsStream>, tonic::Status> {
        match send_endorsements(self, request).await {
            Ok(res) => Ok(tonic::Response::new(res)),
            Err(e) => Err(e.into()),
        }
    }

    /// Handler for send_operations
    async fn send_operations(
        &self,
        request: tonic::Request<tonic::Streaming<grpc::SendOperationsRequest>>,
    ) -> Result<tonic::Response<Self::SendOperationsStream>, tonic::Status> {
        match send_operations(self, request).await {
            Ok(res) => Ok(tonic::Response::new(res)),
            Err(e) => Err(e.into()),
        }
    }

    /// Handler for subscribe on transactions throughput
    async fn subscribe_transactions_throughput(
        &self,
        request: Request<Streaming<GetTransactionsThroughputStreamRequest>>,
    ) -> Result<Response<Self::SubscribeTransactionsThroughputStream>, Status> {
        match subscribe_transactions_throughput(self, request).await {
            Ok(res) => Ok(Response::new(res)),
            Err(e) => Err(e.into()),
        }
    }

    /// Handler for subscribe new operations stream
    async fn subscribe_new_operations(
        &self,
        request: Request<Streaming<SubscribeNewOperationsStreamRequest>>,
    ) -> Result<Response<Self::SubscribeNewOperationsStream>, Status> {
        match subscribe_new_operations(self, request).await {
            Ok(res) => Ok(Response::new(res)),
            Err(e) => Err(e.into()),
        }
    }
}
