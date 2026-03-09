//! LLM provider implementations for Sigil.
//!
//! Implements the `Provider` trait for Anthropic ([`AnthropicProvider`]),
//! OpenRouter ([`OpenRouterProvider`]), and Ollama ([`OllamaProvider`]).
//! Includes embedding support via OpenRouter ([`OpenRouterEmbedder`]),
//! per-model cost estimation ([`estimate_cost`]), and a retry wrapper ([`ReliableProvider`]).

pub mod anthropic;
pub mod embedder;
pub mod ollama;
pub mod openrouter;
pub mod pricing;
pub mod reliable;

pub use anthropic::AnthropicProvider;
pub use embedder::OpenRouterEmbedder;
pub use ollama::OllamaProvider;
pub use openrouter::OpenRouterProvider;
pub use pricing::estimate_cost;
pub use reliable::ReliableProvider;
