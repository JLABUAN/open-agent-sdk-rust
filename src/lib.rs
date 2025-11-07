//! # Open Agent SDK - Rust Implementation
//!
//! A production-ready, streaming-first Rust SDK for building AI agents with local OpenAI-compatible servers.
//!
//! ## Overview
//!
//! This SDK provides a clean, ergonomic API for working with local LLM servers such as:
//! - LM Studio
//! - Ollama
//! - llama.cpp
//! - vLLM
//!
//! ## Key Features
//!
//! - **Zero API Costs**: Run models on your own hardware
//! - **Privacy-First**: All data stays local on your machine
//! - **High Performance**: Native async/await with Tokio runtime
//! - **Streaming Responses**: Real-time token-by-token streaming
//! - **Tool Calling**: Define and execute tools with automatic schema generation
//! - **Lifecycle Hooks**: Intercept and control execution at key points
//! - **Interrupts**: Gracefully cancel long-running operations
//! - **Context Management**: Manual token estimation and history truncation
//! - **Retry Logic**: Exponential backoff with jitter for reliability
//!
//! ## Two Interaction Modes
//!
//! ### 1. Simple Query Function (`query()`)
//! For single-turn interactions without conversation state:
//!
//! ```rust,no_run
//! use open_agent::{query, AgentOptions, ContentBlock};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure the agent with required settings
//!     let options = AgentOptions::builder()
//!         .system_prompt("You are a helpful assistant")
//!         .model("qwen2.5-32b-instruct")
//!         .base_url("http://localhost:1234/v1")
//!         .build()?;
//!
//!     // Send a single query and stream the response
//!     let mut stream = query("What's the capital of France?", &options).await?;
//!
//!     // Process each content block as it arrives
//!     while let Some(block) = stream.next().await {
//!         match block? {
//!             ContentBlock::Text(text_block) => {
//!                 print!("{}", text_block.text);
//!             }
//!             ContentBlock::ToolUse(tool_block) => {
//!                 println!("Tool called: {}", tool_block.name);
//!             }
//!             ContentBlock::ToolResult(_) => {
//!                 // Tool results can be ignored in simple queries
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### 2. Client Object (`Client`)
//! For multi-turn conversations with persistent state:
//!
//! ```rust,no_run
//! use open_agent::{Client, AgentOptions, ContentBlock};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let options = AgentOptions::builder()
//!         .system_prompt("You are a helpful assistant")
//!         .model("qwen2.5-32b-instruct")
//!         .base_url("http://localhost:1234/v1")
//!         .build()?;
//!
//!     // Create a stateful client that maintains conversation history
//!     let mut client = Client::new(options);
//!
//!     // First turn
//!     client.send("What's 2+2?").await?;
//!     while let Some(block) = client.receive().await? {
//!         if let ContentBlock::Text(text) = block {
//!             print!("{}", text.text);
//!         }
//!     }
//!
//!     // Second turn - client remembers previous context
//!     client.send("What about if we multiply that by 3?").await?;
//!     while let Some(block) = client.receive().await? {
//!         if let ContentBlock::Text(text) = block {
//!             print!("{}", text.text);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The SDK is organized into several modules, each with a specific responsibility:
//!
//! - **client**: Core streaming query engine and multi-turn client
//! - **types**: Data structures for messages, content blocks, and configuration
//! - **tools**: Tool definition system with automatic JSON schema generation
//! - **hooks**: Lifecycle event system for intercepting execution
//! - **config**: Provider-specific configuration helpers
//! - **error**: Comprehensive error types and conversions
//! - **context**: Token estimation and message truncation utilities
//! - **retry**: Exponential backoff retry logic with jitter
//! - **utils**: Internal utilities for SSE parsing and tool aggregation

// ============================================================================
// MODULE DECLARATIONS
// ============================================================================
// These modules are private (internal implementation details) unless explicitly
// re-exported through `pub use` statements below.

/// Core client implementation providing streaming queries and stateful conversations.
/// Contains the `query()` function for single-turn queries and `Client` struct
/// for multi-turn conversations with automatic state management.
mod client;

/// Provider configuration helpers for LM Studio, Ollama, llama.cpp, and vLLM.
/// Simplifies endpoint and model name resolution with environment variable support.
mod config;

/// Context window management utilities for token estimation and history truncation.
/// Provides manual control over conversation memory to prevent context overflow.
mod context;

/// Error types and conversions for comprehensive error handling throughout the SDK.
/// Defines the `Error` enum and `Result<T>` type alias used across all public APIs.
mod error;

/// Lifecycle hooks system for intercepting and controlling execution at key points.
/// Enables security gates, audit logging, input/output modification, and compliance checks.
mod hooks;

/// Tool definition and execution system with automatic JSON schema generation.
/// Allows LLMs to call Rust functions with type-safe parameter handling.
mod tools;

/// Core type definitions for messages, content blocks, and agent configuration.
/// Includes builder patterns for ergonomic configuration and OpenAI API serialization.
mod types;

/// Internal utilities for Server-Sent Events (SSE) parsing and tool call aggregation.
/// Handles the low-level details of streaming response parsing.
mod utils;

// ============================================================================
// PUBLIC EXPORTS
// ============================================================================
// These items form the public API of the SDK. Everything else is internal.

/// Retry utilities with exponential backoff and jitter.
/// Made public as a module so users can access retry configuration and functions
/// for their own operations that need retry logic.
pub mod retry;

// --- Core Client API ---

pub use client::{
    /// Stateful multi-turn conversation client with automatic history management.
    /// Use this when you need to maintain conversation context across multiple turns.
    /// Supports tool execution, interrupts, and lifecycle hooks.
    Client,

    /// Simple single-turn query function that returns a stream of content blocks.
    /// Use this for one-off queries without conversation state.
    /// Returns a ContentStream that yields ContentBlock items as they arrive.
    query,
};

// --- Provider Configuration ---

pub use config::{
    /// Enum representing supported LLM providers (LMStudio, Ollama, LlamaCpp, VLLM).
    /// Used to get default base URLs and model names for each provider.
    Provider,

    /// Get the base URL for API requests, with support for environment variable overrides.
    /// Priority: environment variable > provider default > fallback parameter.
    get_base_url,

    /// Get the model name for requests, with optional environment variable override.
    /// Priority: environment variable (if prefer_env=true) > fallback parameter.
    get_model,
};

// --- Context Management ---

pub use context::{
    /// Estimate the number of tokens in a message history using a character-based approximation.
    /// Approximation: ~1 token per 4 characters (70-85% accurate across model families).
    estimate_tokens,

    /// Check if a message history is approaching a token limit.
    /// Returns true if estimated tokens exceed the limit. Useful for proactive truncation.
    is_approaching_limit,

    /// Truncate message history to keep only the most recent turns.
    /// Can optionally preserve the system message regardless of turn count.
    truncate_messages,
};

// --- Error Handling ---

pub use error::{
    /// Comprehensive error type covering HTTP, JSON, API, streaming, and configuration errors.
    /// Implements std::error::Error and provides detailed error context.
    Error,

    /// Type alias for Result<T, Error> used throughout the SDK.
    /// Makes error handling more concise in client code.
    Result,
};

// --- Lifecycle Hooks ---

pub use hooks::{
    /// Constant string identifier for the PreToolUse hook type.
    /// Used internally for hook registration and logging.
    HOOK_PRE_TOOL_USE,

    /// Constant string identifier for the PostToolUse hook type.
    /// Used internally for hook registration and logging.
    HOOK_POST_TOOL_USE,

    /// Constant string identifier for the UserPromptSubmit hook type.
    /// Used internally for hook registration and logging.
    HOOK_USER_PROMPT_SUBMIT,

    /// Decision object returned by hooks to control execution flow.
    /// Can continue, block, or modify inputs/prompts during lifecycle events.
    HookDecision,

    /// Container for registering and managing lifecycle hooks.
    /// Hooks are executed sequentially with the first non-None decision taking effect.
    Hooks,

    /// Event data passed to PostToolUse hooks after tool execution.
    /// Contains tool name, input, ID, result, and full conversation history.
    PostToolUseEvent,

    /// Event data passed to PreToolUse hooks before tool execution.
    /// Contains tool name, input, ID, and full conversation history.
    PreToolUseEvent,

    /// Event data passed to UserPromptSubmit hooks before sending prompts to the API.
    /// Contains the user prompt and full conversation history.
    UserPromptSubmitEvent,
};

// --- Tool System ---

pub use tools::{
    /// Tool definition with name, description, JSON schema, and async handler.
    /// Created using ToolBuilder or the tool() convenience function.
    Tool,

    /// Builder for constructing tools with fluent parameter definition.
    /// Automatically generates JSON schema from parameter types.
    ToolBuilder,

    /// Convenience function to start building a tool with name and description.
    /// Returns a ToolBuilder for adding parameters and handler.
    tool,
};

// --- Core Types ---

pub use types::{
    /// Configuration options for agents, built using the builder pattern.
    /// Contains system prompt, model, base URL, tools, hooks, and execution settings.
    AgentOptions,

    /// Builder for constructing AgentOptions with type-safe validation.
    /// Required fields: system_prompt, model, base_url.
    AgentOptionsBuilder,

    /// Enum representing a unit of content in a message (Text, ToolUse, or ToolResult).
    /// Messages can contain multiple content blocks of different types.
    ContentBlock,

    /// A single message in a conversation with a role and content blocks.
    /// Used to build conversation history and communicate with the LLM.
    Message,

    /// Role of a message participant (System, User, Assistant, or Tool).
    /// Determines how the LLM interprets the message content.
    MessageRole,

    /// Content block containing plain text generated by the model or provided by the user.
    /// Contains a single text field with the content.
    TextBlock,

    /// Content block containing the result of a tool execution.
    /// Includes the tool use ID, content (success or error), and optional error flag.
    ToolResultBlock,

    /// Content block representing a tool call made by the model.
    /// Contains tool name, unique ID, and JSON input parameters.
    ToolUseBlock,
};

// ============================================================================
// CONVENIENCE PRELUDE
// ============================================================================

/// Convenience module containing the most commonly used types and functions.
/// Import with `use open_agent::prelude::*;` to get everything you need for typical usage.
///
/// This includes:
/// - Configuration: AgentOptions, AgentOptionsBuilder
/// - Client: Client, query()
/// - Content: ContentBlock, TextBlock, ToolUseBlock
/// - Tools: Tool, tool()
/// - Hooks: Hooks, HookDecision, hook event types
/// - Errors: Error, Result
pub mod prelude {
    pub use crate::{
        AgentOptions, AgentOptionsBuilder, Client, ContentBlock, Error, HookDecision, Hooks,
        PostToolUseEvent, PreToolUseEvent, Result, TextBlock, Tool, ToolUseBlock,
        UserPromptSubmitEvent, query, tool,
    };
}
