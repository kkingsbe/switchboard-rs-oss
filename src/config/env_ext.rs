//! Environment variable resolution for Config.
//! This module extends Config with env var resolution functionality.

#[cfg(feature = "discord")]
use crate::config::env::{get_env_vars, resolve_env_vars};

impl crate::config::Config {
    /// Resolve environment variable references in configuration values.
    /// This scans for ${VAR} or ${VAR:-default} patterns and replaces
    /// them with values from environment or .env file.
    pub fn resolve_env_vars(&mut self) {
        #[cfg(feature = "discord")]
        {
            let env_vars = get_env_vars();
            self.resolve_discord_config(env_vars);
        }
        
        #[cfg(not(feature = "discord"))]
        {
            // No-op when discord feature is not enabled
        }
    }

    #[cfg(feature = "discord")]
    fn resolve_discord_config(&mut self, env_vars: &std::collections::HashMap<String, String>) {
        // Access the discord field which only exists with discord feature
        if let Some(ref mut discord) = self.discord {
            // Resolve token_env
            let resolved = resolve_env_vars(&discord.token_env, env_vars);
            
            if resolved != discord.token_env {
                // Check if it's a plain env var name
                if is_plain_env_var_name(&resolved) {
                    if let Ok(val) = std::env::var(&resolved) {
                        discord.token_env = val;
                    } else {
                        discord.token_env = resolved;
                    }
                } else {
                    discord.token_env = resolved;
                }
            }

            // Resolve LLM api_key_env
            if let Some(ref mut llm) = discord.llm {
                let resolved_key = resolve_env_vars(&llm.api_key_env, env_vars);
                
                if resolved_key != llm.api_key_env {
                    if is_plain_env_var_name(&resolved_key) {
                        if let Ok(val) = std::env::var(&resolved_key) {
                            llm.api_key_env = val;
                        } else {
                            llm.api_key_env = resolved_key;
                        }
                    } else {
                        llm.api_key_env = resolved_key;
                    }
                }
            }
        }
    }
}

#[cfg(feature = "discord")]
fn is_plain_env_var_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}
