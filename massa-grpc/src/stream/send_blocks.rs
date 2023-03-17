use crate::error::{match_for_io_error, GrpcError};
use crate::service::MassaGrpcService;
use futures_util::StreamExt;
use massa_models::block::{BlockDeserializer, BlockDeserializerArgs, SecureShareBlock};
use massa_models::error::ModelsError;
use massa_models::secure_share::SecureShareDeserializer;
use massa_proto::google::rpc::Status;
use massa_proto::massa::api::v1::{self as grpc, SendBlocksResponse};
use massa_serialization::{DeserializeError, Deserializer};
use std::io::ErrorKind;
use std::pin::Pin;
use tokio::sync::mpsc::Sender;
use tonic::codegen::futures_core;
use tonic::Request;
use tracing::log::{error, warn};

/// type declaration for SendBlockStream
pub type SendBlocksStream = Pin<
    Box<
        dyn futures_core::Stream<Item = Result<SendBlocksResponse, tonic::Status>> + Send + 'static,
    >,
>;

pub(crate) async fn send_blocks(
    grpc: &MassaGrpcService,
    request: Request<tonic::Streaming<grpc::SendBlocksRequest>>,
) -> Result<SendBlocksStream, GrpcError> {
    let consensus_controller = grpc.consensus_controller.clone();
    let mut protocol_sender = grpc.protocol_command_sender.clone();
    let storage = grpc.storage.clone_without_refs();
    let config = grpc.grpc_config.clone();
    let (tx, rx) = tokio::sync::mpsc::channel(config.max_channel_size);
    let mut in_stream = request.into_inner();

    tokio::spawn(async move {
        while let Some(result) = in_stream.next().await {
            match result {
                Ok(req_content) => {
                    let Some(proto_block) = req_content.block else {
                        send_blocks_notify_error(
                            req_content.id.clone(),
                            tx.clone(),
                            tonic::Code::InvalidArgument,
                            "the request payload is empty".to_owned(),
                        ).await;
                        continue;
                    };

                    let args = BlockDeserializerArgs {
                        thread_count: config.thread_count,
                        max_operations_per_block: config.max_operations_per_block,
                        endorsement_count: config.endorsement_count,
                    };

                    let _: Result<(), DeserializeError> =
                        match SecureShareDeserializer::new(BlockDeserializer::new(args))
                            .deserialize::<DeserializeError>(&proto_block.serialized_content)
                        {
                            Ok(tuple) => {
                                let (rest, res_block): (&[u8], SecureShareBlock) = tuple;
                                if rest.is_empty() {
                                    if let Err(e) = res_block
                                        .verify_signature()
                                        .and_then(|_| res_block.content.header.verify_signature())
                                        .map(|_| {
                                            res_block
                                                .content
                                                .header
                                                .content
                                                .endorsements
                                                .iter()
                                                .map(|endorsement| endorsement.verify_signature())
                                                .collect::<Vec<Result<(), ModelsError>>>()
                                        })
                                    {
                                        // Signature error
                                        send_blocks_notify_error(
                                            req_content.id.clone(),
                                            tx.clone(),
                                            tonic::Code::InvalidArgument,
                                            format!("wrong signature: {}", e),
                                        )
                                        .await;
                                        continue;
                                    }

                                    let block_id = res_block.id;
                                    let slot = res_block.content.header.content.slot;
                                    let mut block_storage = storage.clone_without_refs();
                                    block_storage.store_block(res_block.clone());
                                    consensus_controller.register_block(
                                        block_id,
                                        slot,
                                        block_storage.clone(),
                                        false,
                                    );

                                    if let Err(e) =
                                        protocol_sender.integrated_block(block_id, block_storage)
                                    {
                                        send_blocks_notify_error(
                                            req_content.id.clone(),
                                            tx.clone(),
                                            tonic::Code::Internal,
                                            format!("failed to propagate block: {}", e),
                                        )
                                        .await;
                                        // continue ?
                                        continue;
                                    };

                                    let result = grpc::BlockResult {
                                        id: res_block.id.to_string(),
                                    };

                                    if let Err(e) = tx
                                        .send(Ok(SendBlocksResponse {
                                            id: req_content.id.clone(),

                                            result: Some(grpc::send_blocks_response::Result::Ok(
                                                result,
                                            )),
                                        }))
                                        .await
                                    {
                                        error!("failed to send back block response: {}", e);
                                    };
                                } else {
                                    send_blocks_notify_error(
                                        req_content.id.clone(),
                                        tx.clone(),
                                        tonic::Code::InvalidArgument,
                                        "there is data left after operation deserialization"
                                            .to_owned(),
                                    )
                                    .await;
                                }
                                Ok(())
                            }
                            Err(e) => {
                                send_blocks_notify_error(
                                    req_content.id.clone(),
                                    tx.clone(),
                                    tonic::Code::InvalidArgument,
                                    format!("failed to deserialize block: {}", e),
                                )
                                .await;
                                Ok(())
                            }
                        };
                }
                Err(err) => {
                    if let Some(io_err) = match_for_io_error(&err) {
                        if io_err.kind() == ErrorKind::BrokenPipe {
                            warn!("client disconnected, broken pipe: {}", io_err);
                            break;
                        }
                    }
                    error!("{}", err);
                    if let Err(e) = tx.send(Err(err)).await {
                        error!("failed to send back send_blocks error response: {}", e);
                        break;
                    }
                }
            }
        }
    });

    let out_stream = tokio_stream::wrappers::ReceiverStream::new(rx);

    Ok(Box::pin(out_stream) as SendBlocksStream)
}

async fn send_blocks_notify_error(
    id: String,
    sender: Sender<Result<SendBlocksResponse, tonic::Status>>,
    code: tonic::Code,
    error: String,
) {
    error!("{}", error);
    if let Err(e) = sender
        .send(Ok(SendBlocksResponse {
            id,
            result: Some(grpc::send_blocks_response::Result::Error(Status {
                code: code.into(),
                message: error,
                details: Vec::new(),
            })),
        }))
        .await
    {
        error!("failed to send back send_blocks error response: {}", e);
    }
}
