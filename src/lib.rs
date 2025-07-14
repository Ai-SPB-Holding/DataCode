pub mod interpreter;
pub mod value;
pub mod builtins;
pub mod error;
pub mod parser;
pub mod evaluator;
pub mod repl;

// Legacy builtins module for backward compatibility
#[path = "builtins_legacy.rs"]
pub mod builtins_legacy;