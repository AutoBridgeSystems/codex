mod device_code_auth;
mod pkce;
mod server;

pub use device_code_auth::run_device_code_login;
pub use server::LoginServer;
pub use server::ServerOptions;
pub use server::ShutdownHandle;
pub use server::run_login_server;

// Re-export commonly used auth types and helpers from adom-core for compatibility
pub use adom_app_server_protocol::AuthMode;
pub use adom_core::AuthManager;
pub use adom_core::AdomAuth;
pub use adom_core::auth::AuthDotJson;
pub use adom_core::auth::CLIENT_ID;
pub use adom_core::auth::ADOM_API_KEY_ENV_VAR;
pub use adom_core::auth::OPENAI_API_KEY_ENV_VAR;
pub use adom_core::auth::login_with_api_key;
pub use adom_core::auth::logout;
pub use adom_core::auth::save_auth;
pub use adom_core::token_data::TokenData;
