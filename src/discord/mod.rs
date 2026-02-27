//! Discord Concierge module - manages Discord bot integration for AI coding agents.
//!
//! This module provides the core infrastructure for running a Discord bot that serves
//! as a concierge for AI coding agents. It handles Discord gateway connections, API
//! interactions, message processing, LLM integration, and conversation management.
//!
//! # Submodules
//!
//! - [`config`] - Discord bot configuration and settings
//! - [`gateway`] - Discord Gateway connection management
//! - [`api`] - Discord REST API client
//! - [`listener`] - Discord event listener and message handling
//! - [`conversation`] - Conversation state and history management
//! - [`llm`] - LLM integration for generating responses
//! - [`tools`] - Tool execution and result formatting
//! - [`outbox`] - Message queue for async responses
//! - [`security`] - Security utilities (path traversal prevention)
//!
//! # Quick Start
//!
//! ```ignore
//! use switchboard::discord::start_discord_listener;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     start_discord_listener().await?;
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::Result;

pub mod config;

// Re-export config types for convenient access
pub use config::{
    get_discord_channel_id, get_discord_token, get_openrouter_api_key, load_discord_config,
    load_discord_section_from_toml, load_system_prompt, DiscordConfig, DiscordEnvConfig,
    DiscordSection, LlmConfig, DEFAULT_SYSTEM_PROMPT, DISCORD_TOKEN_ENV, OPENROUTER_API_KEY_ENV,
};

// Import resolve_config_value for handling ${VAR} syntax in config
use crate::config::env::resolve_config_value;

/// Security module for Discord tools.
///
/// Provides path traversal prevention and other security utilities
/// for safe file system operations.
pub mod security;

pub mod gateway;

// Re-export gateway types
pub use gateway::{DiscordEvent, DiscordGateway, GatewayCommand, GatewayError, GatewayOpcode};

/// Discord API module.
///
/// Provides a client for making requests to the Discord REST API,
/// including channels, messages, users, and webhooks.
pub mod api;

/// Discord event listener module.
///
/// Handles incoming Discord events, message parsing, and
/// dispatching to appropriate handlers.
pub mod listener;

// Re-export listener types for convenient access
pub use listener::{
    DiscordMessage, DiscordUser, ListenerConfig, MessageHandler, MessageHandlerError,
    ProcessedMessage,
};

/// Conversation management module.
///
/// Manages conversation state, history, and context for
/// ongoing interactions with users.
pub mod conversation;

/// Re-export conversation types for convenient access
pub use conversation::{
    ChatMessage, Conversation, ConversationConfig, ConversationManager, MessageRole, ToolCall,
    ToolFunction,
};

/// LLM integration module.
///
/// Provides integration with Large Language Models for
/// generating responses to user messages.
pub mod llm;

/// Re-export LLM types for convenient access
pub use llm::{
    get_user_error_message, process_with_tools, LlmError, LlmResponse, OpenRouterClient,
    ToolCallResult, ToolExecutor,
};

/// Tools module.
///
/// Defines and executes tools that agents can use to
/// perform actions and return results to users.
pub mod tools;

/// Outbox module.
///
/// Provides a queue for sending messages asynchronously,
/// handling rate limits and retry logic.
pub mod outbox;

/// Environment variable name for Discord channel ID.
pub const DISCORD_CHANNEL_ID_ENV: &str = "DISCORD_CHANNEL_ID";

/// Shared state for the Discord bot event processor.
/// This holds all the components needed to process messages.
struct BotState {
    /// OpenRouter client for LLM interactions
    llm_client: llm::OpenRouterClient,
    /// Conversation manager for per-user conversation state
    conversation_manager: Mutex<conversation::ConversationManager>,
    /// Discord API client for sending messages (wrapped in Arc<Mutex> for concurrent access)
    api_client: Arc<Mutex<api::DiscordApiClient>>,
    /// Bot's user ID (to filter out self-messages)
    bot_user_id: u64,
    /// Channel ID to listen to
    channel_id: String,
    /// System prompt for the LLM
    system_prompt: String,
}

impl BotState {
    /// Create a new BotState.
    fn new(
        llm_client: llm::OpenRouterClient,
        conversation_manager: conversation::ConversationManager,
        api_client: api::DiscordApiClient,
        bot_user_id: u64,
        channel_id: String,
        system_prompt: String,
    ) -> Self {
        Self {
            llm_client,
            conversation_manager: Mutex::new(conversation_manager),
            api_client: Arc::new(Mutex::new(api_client)),
            bot_user_id,
            channel_id,
            system_prompt,
        }
    }
}

/// Starts the Discord listener bot.
///
/// This function initializes the Discord bot and begins listening
/// for incoming messages and events. Configuration is loaded from
/// environment variables:
/// - DISCORD_TOKEN: Discord bot token
/// - OPENROUTER_API_KEY: OpenRouter API key for LLM
/// - DISCORD_CHANNEL_ID: Discord channel ID to listen to
///
/// # Arguments
///
/// * `system_prompt_file` - Optional path to a custom system prompt file
///
/// # Returns
///
/// * `Result<()>` - Ok on successful start, Error on failure
///
/// # Example
///
/// ```ignore
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     if let Err(e) = switchboard::discord::start_discord_listener(None).await {
///         eprintln!("Failed to start Discord listener: {}", e);
///         std::process::exit(1);
///     }
/// }
/// ```
pub async fn start_discord_listener(system_prompt_file: Option<String>) -> Result<()> {
    start_discord_listener_with_shutdown(None, system_prompt_file).await
}

/// Starts the Discord listener bot with graceful shutdown support.
///
/// This function initializes the Discord bot and begins listening
/// for incoming messages and events. Configuration is loaded from
/// environment variables:
/// - DISCORD_TOKEN: Discord bot token
/// - OPENROUTER_API_KEY: OpenRouter API key for LLM
/// - DISCORD_CHANNEL_ID: Discord channel ID to listen to
///
/// # Arguments
///
/// * `shutdown_rx` - Optional broadcast receiver for shutdown signal. When provided and received,
///   all background tasks (outbox poller, event processor, gateway) will stop.
/// * `system_prompt_file` - Optional path to a custom system prompt file
///
/// # Returns
///
/// * `Result<()>` - Ok on successful start, Error on failure
///
/// # Example
///
/// ```ignore
/// use tokio::sync::broadcast;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
///     if let Err(e) = switchboard::discord::start_discord_listener_with_shutdown(shutdown_rx, None).await {
///         eprintln!("Failed to start Discord listener: {}", e);
///         std::process::exit(1);
///     }
///     // To stop the Discord listener:
///     let _ = shutdown_tx.send(());
/// }
/// ```
pub async fn start_discord_listener_with_shutdown(
    shutdown_rx: Option<tokio::sync::broadcast::Receiver<()>>,
    system_prompt_file: Option<String>,
) -> Result<()> {
    eprintln!("[DEBUG] Discord: start_discord_listener_with_shutdown called");
    tracing::info!("Discord listener starting...");

    // Try to load Discord configuration from TOML file first
    let toml_config = match load_discord_section_from_toml("./switchboard.toml") {
        Ok(Some(config)) => {
            tracing::info!("Loaded Discord config from switchboard.toml");
            Some(config)
        }
        Ok(None) => {
            tracing::info!("No [discord] section found in switchboard.toml, will use env vars");
            None
        }
        Err(e) => {
            tracing::warn!("Failed to load switchboard.toml: {}, will use env vars", e);
            None
        }
    };

    // Determine configuration values with fallback: TOML -> env vars

    // Discord token: use resolve_config_value to handle ${VAR} syntax
    let discord_token = if let Some(ref toml_cfg) = toml_config {
        // Use resolve_config_value which handles ${VAR} syntax and checks switchboard.env
        let token = resolve_config_value(&toml_cfg.token_env);
        if token.is_empty() {
            return Err(anyhow::anyhow!(
                "Missing Discord token: {} not found in switchboard.env or environment",
                toml_cfg.token_env
            ));
        }
        tracing::info!("Discord token resolved from config");
        token
    } else {
        // Fall back to env var
        let env_config = load_discord_config()
            .map_err(|e| anyhow::anyhow!("Failed to load Discord config: {}", e))?;
        tracing::info!("Loaded Discord token from environment");
        env_config.discord_token
    };

    // Channel ID: use TOML value if present, else try env var
    let channel_id = if let Some(ref toml_cfg) = toml_config {
        // Use resolve_config_value to handle ${VAR} syntax
        let channel = resolve_config_value(&toml_cfg.channel_id);
        if !channel.is_empty() {
            tracing::info!("Using channel ID from TOML config");
            channel
        } else {
            get_discord_channel_id()
                .map_err(|e| anyhow::anyhow!("Failed to get Discord channel ID: {}", e))?
        }
    } else {
        get_discord_channel_id()
            .map_err(|e| anyhow::anyhow!("Failed to get Discord channel ID: {}", e))?
    };

    tracing::info!("Discord channel ID: {}", channel_id);

    // OpenRouter API key: use TOML api_key_env if present, else try env var
    let (openrouter_api_key, model, max_tokens) = if let Some(ref toml_cfg) = toml_config {
        if let Some(ref llm_cfg) = toml_cfg.llm {
            // Use resolve_config_value to handle ${VAR} syntax
            let api_key = resolve_config_value(&llm_cfg.api_key_env);
            if api_key.is_empty() {
                return Err(anyhow::anyhow!(
                    "Missing OpenRouter API key: {} not found in switchboard.env or environment",
                    llm_cfg.api_key_env
                ));
            }
            tracing::info!("OpenRouter API key resolved from config");
            (api_key, llm_cfg.model.clone(), llm_cfg.max_tokens)
        } else {
            // No LLM config in TOML, use env var for API key and defaults for model/max_tokens
            let api_key = get_openrouter_api_key()
                .map_err(|e| anyhow::anyhow!("Failed to get OpenRouter API key: {}", e))?;
            (api_key, "anthropic/claude-sonnet-4".to_string(), 1024)
        }
    } else {
        // No TOML config, use env var for API key and defaults
        let api_key = get_openrouter_api_key()
            .map_err(|e| anyhow::anyhow!("Failed to get OpenRouter API key: {}", e))?;
        (api_key, "anthropic/claude-sonnet-4".to_string(), 1024)
    };

    // Create LLM client with resolved model and max tokens
    let llm_client = llm::OpenRouterClient::new(openrouter_api_key, model, max_tokens);

    // Create conversation manager with default config
    let conversation_config = conversation::ConversationConfig::default();
    let conversation_manager = conversation::ConversationManager::new(conversation_config);

    // Create Discord API client
    let api_client = api::DiscordApiClient::new(discord_token.clone());

    // OUTBOX POLLER DISABLED - Clone the token for the outbox poller
    // let token_for_outbox = discord_token.clone();
    // // Initialize Discord API client for outbox poller
    // let outbox_api_client = api::DiscordApiClient::new(token_for_outbox);

    // OUTBOX POLLER DISABLED - Uncomment to re-enable auto-posting
    // let mut outbox_poller =
    //     outbox::OutboxPoller::with_defaults(channel_id.clone(), outbox_api_client);

    // Set up Discord Gateway connection
    let gateway_token = discord_token.clone();

    // Create a channel for gateway events
    let (event_sender, event_receiver) = tokio::sync::mpsc::channel::<DiscordEvent>(100);

    // Create the bot state - we'll get the bot_user_id after Ready event
    // For now, use 0 as placeholder until we get the actual user ID
    // Load system prompt from file or use default
    let system_prompt = load_system_prompt(system_prompt_file.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to load system prompt: {}", e))?;

    let bot_state = Arc::new(Mutex::new(BotState::new(
        llm_client,
        conversation_manager,
        api_client,
        0, // Will be updated when Ready event is received
        channel_id.clone(),
        system_prompt,
    )));

    // Clone the state for the event processor
    let state_for_processor = Arc::clone(&bot_state);

    // Intents: GUILD_MESSAGES (512) + DIRECT_MESSAGES (4096) + MESSAGE_CONTENT (16384) = 21504
    // MESSAGE_CONTENT (16384) is required to read message content
    // GUILD_MESSAGES allows receiving messages from guild channels
    // DIRECT_MESSAGES allows receiving messages from DMs
    // NOTE: MESSAGE_CONTENT is 16384, NOT 1024 (1024 is GUILD_MESSAGE_REACTIONS)
    // Using GUILD_MESSAGES | DIRECT_MESSAGES | MESSAGE_CONTENT to receive all message types
    const DEFAULT_INTENTS: u32 = 512 | 4096 | 16384;

    // Get intents from config if provided, otherwise use default
    let gateway_intents = if let Some(ref toml_cfg) = toml_config {
        if let Some(intents) = toml_cfg.intents {
            eprintln!(
                "[DEBUG] Discord Gateway: Using custom intents from config: {}",
                intents
            );
            intents
        } else {
            eprintln!(
                "[DEBUG] Discord Gateway: Using default intents: {}",
                DEFAULT_INTENTS
            );
            DEFAULT_INTENTS
        }
    } else {
        eprintln!(
            "[DEBUG] Discord Gateway: Using default intents (no config): {}",
            DEFAULT_INTENTS
        );
        DEFAULT_INTENTS
    };

    eprintln!("[DEBUG] Discord: about to check shutdown_rx");

    match shutdown_rx {
        Some(shutdown_rx) => {
            // OUTBOX POLLER DISABLED - Outbox poller removed
            // let outbox_shutdown_rx = shutdown_rx.resubscribe();
            // tokio::spawn(async move {
            //     outbox_poller.start_with_shutdown(outbox_shutdown_rx).await;
            // });
            // tracing::info!("Outbox poller spawned as background task with shutdown support");

            // Subscribe to shutdown channel for event processor
            let processor_shutdown_rx = shutdown_rx.resubscribe();

            // Spawn a task to process Discord events with shutdown support
            tokio::spawn(async move {
                process_gateway_events(event_receiver, state_for_processor, processor_shutdown_rx)
                    .await;
            });

            // Create a oneshot channel for gateway shutdown
            let (gateway_shutdown_tx, gateway_shutdown_rx) = tokio::sync::oneshot::channel();

            // Subscribe to shutdown channel for the gateway shutdown forwarder task
            let mut gateway_shutdown_forward_rx = shutdown_rx.resubscribe();

            // Spawn a task that forwards broadcast shutdown to gateway's oneshot
            tokio::spawn(async move {
                let _ = gateway_shutdown_forward_rx.recv().await;
                tracing::info!("Gateway shutdown signal received, sending to gateway");
                let _ = gateway_shutdown_tx.send(());
            });

            // Spawn the Discord Gateway as a background task
            tokio::spawn(async move {
                // Debug: Log token prefix and intents
                let token_prefix = if gateway_token.len() > 10 {
                    format!(
                        "{}...{}",
                        &gateway_token[..5],
                        &gateway_token[gateway_token.len() - 5..]
                    )
                } else {
                    "[token too short]".to_string()
                };
                eprintln!(
                    "[DEBUG] Discord Gateway: Token prefix: {}, Intents: {}",
                    token_prefix, gateway_intents
                );

                // Create the gateway instance
                let mut gateway =
                    DiscordGateway::new(gateway_token.clone(), gateway_intents, event_sender);

                eprintln!("[DEBUG] Discord Gateway: About to connect...");
                tracing::info!("Discord Gateway: Starting connection...");

                // Connect to the gateway (pass the shutdown receiver to listen for shutdown)
                match gateway.connect_with_shutdown(gateway_shutdown_rx).await {
                    Ok(_) => {
                        tracing::info!("Discord Gateway connection closed normally");
                    }
                    Err(e) => {
                        tracing::error!("Discord Gateway connection error: {}", e);
                    }
                }
            });
        }
        None => {
            // OUTBOX POLLER DISABLED - Outbox poller removed
            // tokio::spawn(async move {
            //     outbox_poller.start().await;
            // });
            // tracing::info!("Outbox poller spawned as background task");

            // Spawn a task to process Discord events without shutdown support
            tokio::spawn(async move {
                process_gateway_events_no_shutdown(event_receiver, state_for_processor).await;
            });

            // Spawn the Discord Gateway as a background task
            tokio::spawn(async move {
                // Debug: Log token prefix and intents
                let token_prefix = if gateway_token.len() > 10 {
                    format!(
                        "{}...{}",
                        &gateway_token[..5],
                        &gateway_token[gateway_token.len() - 5..]
                    )
                } else {
                    "[token too short]".to_string()
                };
                eprintln!(
                    "[DEBUG] Discord Gateway: Token prefix: {}, Intents: {}",
                    token_prefix, gateway_intents
                );

                // Create the gateway instance
                let mut gateway = DiscordGateway::new(gateway_token, gateway_intents, event_sender);

                eprintln!("[DEBUG] Discord Gateway: About to connect (no shutdown)...");
                tracing::info!("Discord Gateway: Starting connection (no shutdown)...");

                // Connect to the gateway - use a channel that never signals
                // (use a receiver that's never populated, which is the opposite of the broken code)
                let (_tx, rx) = tokio::sync::oneshot::channel();
                match gateway.connect_with_shutdown(rx).await {
                    Ok(_) => {
                        tracing::info!("Discord Gateway connection closed normally");
                    }
                    Err(e) => {
                        tracing::error!("Discord Gateway connection error: {}", e);
                    }
                }
            });
        }
    }

    tracing::info!("Discord Gateway spawned as background task");

    tracing::info!("Discord listener started successfully");

    // Return Ok to indicate successful start
    // The background tasks will continue running
    Ok(())
}

/// Process Discord events from the gateway receiver.
///
/// This function runs as a background task, receiving events from the
/// Discord Gateway and processing them appropriately.
///
/// # Arguments
///
/// * `event_receiver` - The channel receiver for Discord events
/// * `state` - Shared bot state for processing messages
/// * `shutdown_rx` - Broadcast receiver for shutdown signal
async fn process_gateway_events(
    mut event_receiver: tokio::sync::mpsc::Receiver<DiscordEvent>,
    state: Arc<Mutex<BotState>>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) {
    tracing::info!("Event processor task started");

    loop {
        tokio::select! {
            event = event_receiver.recv() => {
                match event {
                    Some(event) => {
                        match event {
                            DiscordEvent::Ready {
                                user_id,
                                session_id,
                            } => {
                                tracing::info!(
                                    "DiscordEvent: Ready (user_id: {}, session_id: {})",
                                    user_id,
                                    session_id
                                );
                                // Update the bot user ID in state
                                let mut state_guard = state.lock().await;
                                if let Ok(bot_id) = user_id.parse::<u64>() {
                                    state_guard.bot_user_id = bot_id;
                                    tracing::info!("Bot user ID set to: {}", bot_id);
                                }
                            }
                            DiscordEvent::MessageCreate {
                                channel_id,
                                author_id,
                                content,
                                message_id,
                                guild_id,
                            } => {
                                tracing::info!(
                                    "RECEIVED: DiscordEvent::MessageCreate (channel: {}, author: {}, message_id: {}, guild: {:?})",
                                    channel_id,
                                    author_id,
                                    message_id,
                                    guild_id
                                );
                                tracing::debug!(
                                    "Message content: {}",
                                    content.chars().take(100).collect::<String>()
                                );

                                // Process the message with the LLM
                                let state_clone = Arc::clone(&state);
                                tokio::spawn(async move {
                                    if let Err(e) = handle_message_create_event(
                                        state_clone,
                                        channel_id.clone(),
                                        author_id.clone(),
                                        content.clone(),
                                        message_id.clone(),
                                    )
                                    .await
                                    {
                                        tracing::error!("Error handling message: {}", e);
                                    }
                                });
                            }
                            DiscordEvent::MessageDelete {
                                message_id,
                                channel_id,
                                guild_id,
                            } => {
                                tracing::info!(
                                    "DiscordEvent: MessageDelete (message_id: {}, channel: {}, guild: {:?})",
                                    message_id,
                                    channel_id,
                                    guild_id
                                );
                            }
                            DiscordEvent::GuildCreate { guild_id } => {
                                tracing::info!("DiscordEvent: GuildCreate (guild_id: {})", guild_id);
                            }
                            DiscordEvent::Resumed => {
                                tracing::info!("DiscordEvent: Resumed");
                            }
                            DiscordEvent::InvalidSession => {
                                tracing::warn!("DiscordEvent: InvalidSession");
                            }
                            DiscordEvent::HeartbeatAck => {
                                tracing::debug!("DiscordEvent: HeartbeatAck");
                            }
                            DiscordEvent::Other(_) => {
                                tracing::debug!("DiscordEvent: Other");
                            }
                        }
                    }
                    None => {
                        // Channel closed, exit the loop
                        tracing::info!("Event receiver channel closed, shutting down event processor");
                        break;
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("Event processor received shutdown signal");
                break;
            }
        }
    }

    tracing::info!("Event processor task stopped");
}

/// Process Discord events from the gateway receiver (without shutdown support).
///
/// This function runs as a background task, receiving events from the
/// Discord Gateway and processing them appropriately. This is the legacy version
/// used when no shutdown receiver is provided.
///
/// # Arguments
///
/// * `event_receiver` - The channel receiver for Discord events
/// * `state` - Shared bot state for processing messages
async fn process_gateway_events_no_shutdown(
    mut event_receiver: tokio::sync::mpsc::Receiver<DiscordEvent>,
    state: Arc<Mutex<BotState>>,
) {
    tracing::info!("Event processor task started (no shutdown support)");

    while let Some(event) = event_receiver.recv().await {
        match event {
            DiscordEvent::Ready {
                user_id,
                session_id,
            } => {
                tracing::info!(
                    "DiscordEvent: Ready (user_id: {}, session_id: {})",
                    user_id,
                    session_id
                );
                // Update the bot user ID in state
                let mut state_guard = state.lock().await;
                if let Ok(bot_id) = user_id.parse::<u64>() {
                    state_guard.bot_user_id = bot_id;
                    tracing::info!("Bot user ID set to: {}", bot_id);
                }
            }
            DiscordEvent::MessageCreate {
                channel_id,
                author_id,
                content,
                message_id,
                guild_id,
            } => {
                tracing::info!(
                    "DiscordEvent: MessageCreate (channel: {}, author: {}, message_id: {}, guild: {:?})",
                    channel_id,
                    author_id,
                    message_id,
                    guild_id
                );
                tracing::debug!(
                    "Message content: {}",
                    content.chars().take(100).collect::<String>()
                );

                // Process the message with the LLM
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_message_create_event(
                        state_clone,
                        channel_id.clone(),
                        author_id.clone(),
                        content.clone(),
                        message_id.clone(),
                    )
                    .await
                    {
                        tracing::error!("Error handling message: {}", e);
                    }
                });
            }
            DiscordEvent::MessageDelete {
                message_id,
                channel_id,
                guild_id,
            } => {
                tracing::info!(
                    "DiscordEvent: MessageDelete (message_id: {}, channel: {}, guild: {:?})",
                    message_id,
                    channel_id,
                    guild_id
                );
            }
            DiscordEvent::GuildCreate { guild_id } => {
                tracing::info!("DiscordEvent: GuildCreate (guild_id: {})", guild_id);
            }
            DiscordEvent::Resumed => {
                tracing::info!("DiscordEvent: Resumed");
            }
            DiscordEvent::InvalidSession => {
                tracing::warn!("DiscordEvent: InvalidSession");
            }
            DiscordEvent::HeartbeatAck => {
                tracing::debug!("DiscordEvent: HeartbeatAck");
            }
            DiscordEvent::Other(_) => {
                tracing::debug!("DiscordEvent: Other");
            }
        }
    }

    tracing::info!("Event processor task ended (receiver closed)");
}

/// Handle a MessageCreate event by processing with LLM and sending response.
async fn handle_message_create_event(
    state: Arc<Mutex<BotState>>,
    channel_id: String,
    author_id: String,
    content: String,
    _message_id: String,
) -> anyhow::Result<()> {
    // Get the state
    let state_guard = state.lock().await;

    // Parse author ID
    let author_id_num: u64 = match author_id.parse() {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("Failed to parse author ID: {}", e);
            return Ok(());
        }
    };

    // Check if message is from the bot itself
    if author_id_num == state_guard.bot_user_id {
        tracing::debug!("Ignoring message from bot user");
        return Ok(());
    }

    // Check if message is from the configured channel
    let channel_id_num: u64 = match channel_id.parse() {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("Failed to parse channel ID: {}", e);
            return Ok(());
        }
    };

    let target_channel_id: u64 = match state_guard.channel_id.parse() {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("Failed to parse target channel ID: {}", e);
            return Ok(());
        }
    };

    if channel_id_num != target_channel_id {
        tracing::debug!(
            "Ignoring message from wrong channel {} (expected {})",
            channel_id_num,
            target_channel_id
        );
        return Ok(());
    }

    // Skip empty messages
    if content.trim().is_empty() {
        tracing::debug!("Message content is empty, ignoring");
        return Ok(());
    }

    tracing::info!(
        "Processing message from channel {}: {}",
        channel_id,
        content.chars().take(100).collect::<String>()
    );

    // Get user ID string for conversation management
    let user_id_str = author_id.clone();

    // Add user message to conversation
    let mut conv_manager = state_guard.conversation_manager.lock().await;
    conv_manager.add_user_message(&user_id_str, &content);

    // Get messages for LLM
    let messages = conv_manager
        .get_messages_for_llm(&user_id_str, &state_guard.system_prompt)
        .unwrap_or_else(|| vec![conversation::ChatMessage::user(&content)]);

    // Get LLM client reference
    let llm_client = &state_guard.llm_client;

    // Call the LLM while holding the lock (to avoid borrow issues)
    let llm_response = call_llm(llm_client, messages).await;

    // Process the response
    let response_text = match llm_response {
        Ok(text) => {
            // Add assistant response to conversation
            conv_manager.add_assistant_message(&user_id_str, &text);
            text
        }
        Err(e) => {
            tracing::error!("LLM error: {}", e);
            let error_msg = llm::get_user_error_message(&e);
            // Add error message to conversation
            conv_manager.add_assistant_message(&user_id_str, &error_msg);
            error_msg
        }
    };

    // Send the response to Discord
    let mut api_client = state_guard.api_client.lock().await;
    match api_client
        .send_message_chunked(&channel_id, &response_text)
        .await
    {
        Ok(_) => {
            tracing::info!("Sent response to channel {}", channel_id);
        }
        Err(e) => {
            tracing::error!("Failed to send message to Discord: {}", e);
        }
    }

    Ok(())
}

/// Call the LLM with the given messages.
async fn call_llm(
    client: &llm::OpenRouterClient,
    messages: Vec<conversation::ChatMessage>,
) -> Result<String, llm::LlmError> {
    let mut messages = messages;

    // Get tools from the tools schema
    let tools_json = tools::tools_schema();
    let tools = llm::tools_schema_to_definitions(&tools_json);

    // Create a real tool executor
    let executor = DiscordToolExecutor;

    // Use process_with_tools with the tools
    llm::process_with_tools(client, &mut messages, &tools, &executor).await
}

/// Real tool executor that actually executes Discord agent tools.
struct DiscordToolExecutor;

impl llm::ToolExecutor for DiscordToolExecutor {
    fn execute(&self, name: &str, arguments: &str) -> Result<String, String> {
        // Parse the tool from LLM format
        let tool = tools::parse_tool_from_llm(name, arguments).map_err(|e| e.to_string())?;

        // Execute the tool and return the result
        tools::execute_tool(tool).map_err(|e| e.to_string())
    }
}
