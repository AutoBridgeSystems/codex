use adom_core::AdomAuth;
use adom_core::ConversationManager;
use adom_core::protocol::EventMsg;
use adom_core::protocol::Op;
use adom_core::protocol_config_types::ReasoningEffort;
use core_test_support::load_default_config_for_test;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

const CONFIG_TOML: &str = "config.toml";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_turn_context_does_not_persist_when_config_exists() {
    let adom_home = TempDir::new().unwrap();
    let config_path = adom_home.path().join(CONFIG_TOML);
    let initial_contents = "model = \"gpt-4o\"\n";
    tokio::fs::write(&config_path, initial_contents)
        .await
        .expect("seed config.toml");

    let mut config = load_default_config_for_test(&adom_home);
    config.model = "gpt-4o".to_string();

    let conversation_manager =
        ConversationManager::with_auth(AdomAuth::from_api_key("Test API Key"));
    let adom = conversation_manager
        .new_conversation(config)
        .await
        .expect("create conversation")
        .conversation;

    adom
        .submit(Op::OverrideTurnContext {
            cwd: None,
            approval_policy: None,
            sandbox_policy: None,
            model: Some("o3".to_string()),
            effort: Some(Some(ReasoningEffort::High)),
            summary: None,
        })
        .await
        .expect("submit override");

    adom.submit(Op::Shutdown).await.expect("request shutdown");
    wait_for_event(&adom, |ev| matches!(ev, EventMsg::ShutdownComplete)).await;

    let contents = tokio::fs::read_to_string(&config_path)
        .await
        .expect("read config.toml after override");
    assert_eq!(contents, initial_contents);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_turn_context_does_not_create_config_file() {
    let adom_home = TempDir::new().unwrap();
    let config_path = adom_home.path().join(CONFIG_TOML);
    assert!(
        !config_path.exists(),
        "test setup should start without config"
    );

    let config = load_default_config_for_test(&adom_home);

    let conversation_manager =
        ConversationManager::with_auth(AdomAuth::from_api_key("Test API Key"));
    let adom = conversation_manager
        .new_conversation(config)
        .await
        .expect("create conversation")
        .conversation;

    adom
        .submit(Op::OverrideTurnContext {
            cwd: None,
            approval_policy: None,
            sandbox_policy: None,
            model: Some("o3".to_string()),
            effort: Some(Some(ReasoningEffort::Medium)),
            summary: None,
        })
        .await
        .expect("submit override");

    adom.submit(Op::Shutdown).await.expect("request shutdown");
    wait_for_event(&adom, |ev| matches!(ev, EventMsg::ShutdownComplete)).await;

    assert!(
        !config_path.exists(),
        "override should not create config.toml"
    );
}
