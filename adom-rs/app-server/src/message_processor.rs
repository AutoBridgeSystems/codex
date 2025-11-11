use std::path::PathBuf;

use crate::adom_message_processor::AdomMessageProcessor;
use crate::error_code::INVALID_REQUEST_ERROR_CODE;
use crate::outgoing_message::OutgoingMessageSender;
use adom_app_server_protocol::ClientInfo;
use adom_app_server_protocol::ClientRequest;
use adom_app_server_protocol::InitializeResponse;

use adom_app_server_protocol::JSONRPCError;
use adom_app_server_protocol::JSONRPCErrorError;
use adom_app_server_protocol::JSONRPCNotification;
use adom_app_server_protocol::JSONRPCRequest;
use adom_app_server_protocol::JSONRPCResponse;
use adom_core::AuthManager;
use adom_core::ConversationManager;
use adom_core::config::Config;
use adom_core::default_client::USER_AGENT_SUFFIX;
use adom_core::default_client::get_adom_user_agent;
use adom_feedback::AdomFeedback;
use adom_protocol::protocol::SessionSource;
use std::sync::Arc;

pub(crate) struct MessageProcessor {
    outgoing: Arc<OutgoingMessageSender>,
    adom_message_processor: AdomMessageProcessor,
    initialized: bool,
}

impl MessageProcessor {
    /// Create a new `MessageProcessor`, retaining a handle to the outgoing
    /// `Sender` so handlers can enqueue messages to be written to stdout.
    pub(crate) fn new(
        outgoing: OutgoingMessageSender,
        adom_linux_sandbox_exe: Option<PathBuf>,
        config: Arc<Config>,
        feedback: AdomFeedback,
    ) -> Self {
        let outgoing = Arc::new(outgoing);
        let auth_manager = AuthManager::shared(
            config.adom_home.clone(),
            false,
            config.cli_auth_credentials_store_mode,
        );
        let conversation_manager = Arc::new(ConversationManager::new(
            auth_manager.clone(),
            SessionSource::VSCode,
        ));
        let adom_message_processor = AdomMessageProcessor::new(
            auth_manager,
            conversation_manager,
            outgoing.clone(),
            adom_linux_sandbox_exe,
            config,
            feedback,
        );

        Self {
            outgoing,
            adom_message_processor,
            initialized: false,
        }
    }

    pub(crate) async fn process_request(&mut self, request: JSONRPCRequest) {
        let request_id = request.id.clone();
        let request_json = match serde_json::to_value(&request) {
            Ok(request_json) => request_json,
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: format!("Invalid request: {err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let adom_request = match serde_json::from_value::<ClientRequest>(request_json) {
            Ok(adom_request) => adom_request,
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: format!("Invalid request: {err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        match adom_request {
            // Handle Initialize internally so AdomMessageProcessor does not have to concern
            // itself with the `initialized` bool.
            ClientRequest::Initialize { request_id, params } => {
                if self.initialized {
                    let error = JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message: "Already initialized".to_string(),
                        data: None,
                    };
                    self.outgoing.send_error(request_id, error).await;
                    return;
                } else {
                    let ClientInfo {
                        name,
                        title: _title,
                        version,
                    } = params.client_info;
                    let user_agent_suffix = format!("{name}; {version}");
                    if let Ok(mut suffix) = USER_AGENT_SUFFIX.lock() {
                        *suffix = Some(user_agent_suffix);
                    }

                    let user_agent = get_adom_user_agent();
                    let response = InitializeResponse { user_agent };
                    self.outgoing.send_response(request_id, response).await;

                    self.initialized = true;
                    return;
                }
            }
            _ => {
                if !self.initialized {
                    let error = JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message: "Not initialized".to_string(),
                        data: None,
                    };
                    self.outgoing.send_error(request_id, error).await;
                    return;
                }
            }
        }

        self.adom_message_processor
            .process_request(adom_request)
            .await;
    }

    pub(crate) async fn process_notification(&self, notification: JSONRPCNotification) {
        // Currently, we do not expect to receive any notifications from the
        // client, so we just log them.
        tracing::info!("<- notification: {:?}", notification);
    }

    /// Handle a standalone JSON-RPC response originating from the peer.
    pub(crate) async fn process_response(&mut self, response: JSONRPCResponse) {
        tracing::info!("<- response: {:?}", response);
        let JSONRPCResponse { id, result, .. } = response;
        self.outgoing.notify_client_response(id, result).await
    }

    /// Handle an error object received from the peer.
    pub(crate) fn process_error(&mut self, err: JSONRPCError) {
        tracing::error!("<- error: {:?}", err);
    }
}
