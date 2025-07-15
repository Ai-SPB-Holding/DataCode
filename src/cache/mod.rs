// Модуль кэширования для DataCode

pub mod memoization;

pub use memoization::{
    OperationCache,
    TableId,
    FilterExpr,
    CacheEntry,
    CacheStats
};
