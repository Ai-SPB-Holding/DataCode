// Модуль кэширования для DataCode

pub mod memoization;

pub use memoization::{
    OperationCache,
    FunctionCache,
    TableId,
    FilterExpr,
    CacheEntry,
    CacheStats
};
