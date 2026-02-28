use std::env;

#[cfg(feature = "discord")]
use crate::config::env::resolve_config_value;
#[cfg(feature = "discord")]
use crate::discord::config::LlmConfig;

/// Check if Discord configuration is present in environment variables or switchboard.toml
///
/// First checks for environment variables, then falls back to switchboard.toml config.
/// Returns true only if Discord can be configured.
#[cfg(feature = "discord")]
pub fn is_discord_configured() -> bool {
    // First check if environment variables are already set
    if std::env::var("DISCORD_TOKEN").is_ok()
        && std::env::var("OPENROUTER_API_KEY").is_ok()
        && std::env::var("DISCORD_CHANNEL_ID").is_ok()
    {
        tracing::debug!("Discord configured via environment variables");
        return true;
    }

    // Try to load Discord config from switchboard.toml
    match load_discord_config_from_toml("./switchboard.toml") {
        Ok(full_config) => {
            if full_config.enabled {
                tracing::debug!("Discord config loaded from switchboard.toml: enabled={}, token_env={}, channel_id={}", 
                    full_config.enabled, full_config.token_env, full_config.channel_id);

                // Handle token: use resolve_config_value to handle ${VAR} syntax
                let token = resolve_config_value(&full_config.token_env);

                if !token.is_empty() {
                    env::set_var("DISCORD_TOKEN", &token);
                }

                // Set channel ID
                let channel_id = resolve_config_value(&full_config.channel_id);
                if !channel_id.is_empty() {
                    env::set_var("DISCORD_CHANNEL_ID", &channel_id);
                }

                // Handle LLM API key
                let api_key = if let Some(llm_config) = &full_config.llm {
                    resolve_config_value(&llm_config.api_key_env)
                } else {
                    String::new()
                };

                if !api_key.is_empty() {
                    env::set_var("OPENROUTER_API_KEY", &api_key);
                }

                // Verify we now have all required env vars
                let configured = std::env::var("DISCORD_TOKEN").is_ok()
                    && std::env::var("OPENROUTER_API_KEY").is_ok()
                    && std::env::var("DISCORD_CHANNEL_ID").is_ok();

                tracing::debug!(
                    "Discord env vars set: token={}, api_key={}, channel_id={}",
                    std::env::var("DISCORD_TOKEN").is_ok(),
                    std::env::var("OPENROUTER_API_KEY").is_ok(),
                    std::env::var("DISCORD_CHANNEL_ID").is_ok()
                );

                configured
            } else {
                tracing::debug!("Discord disabled in switchboard.toml");
                false
            }
        }
        Err(e) => {
            tracing::debug!("Failed to load Discord config from switchboard.toml: {}", e);
            false
        }
    }
}

/// Full Discord configuration including LLM settings
#[cfg(feature = "discord")]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DiscordFullConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub token_env: String,
    #[serde(default)]
    pub channel_id: String,
    #[serde(default)]
    pub llm: Option<LlmConfig>,
}

/// Discord configuration specifically for the [discord] section in switchboard.toml
#[cfg(feature = "discord")]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DiscordTomlSection {
    #[serde(default)]
    pub discord: Option<DiscordFullConfig>,
}

/// Load Discord configuration from switchboard.toml file
#[cfg(feature = "discord")]
pub fn load_discord_config_from_toml(
    path: &str,
) -> Result<DiscordFullConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;

    // First try to parse as a struct that contains [discord] section
    if let Ok(section_config) = toml::from_str::<DiscordTomlSection>(&content) {
        if let Some(discord_config) = section_config.discord {
            tracing::debug!("Successfully parsed [discord] section from switchboard.toml");
            return Ok(discord_config);
        }
    }

    // Fallback: try parsing as DiscordFullConfig at root level
    let config: DiscordFullConfig = toml::from_str(&content)?;
    Ok(config)
}
