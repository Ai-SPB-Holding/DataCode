/*!
# DataCode Built-in Functions Module

This module provides a modular architecture for DataCode's built-in functions,
organized by functionality for better maintainability and extensibility.

## Module Structure

- **system**: System utilities (now, print, getcwd, isinstance)
- **file**: File operations (path, list_files, read_file, analyze_csv)
- **math**: Mathematical functions (abs, sqrt, pow, min, max, round, div)
- **array**: Array operations (length, push, pop, sort, unique, sum, average)
- **string**: String manipulation (split, join, trim, upper, lower, contains)
- **table**: Core table operations (table, show_table, table_info, table_select)
- **filter**: Table filtering (table_where, table_filter, table_distinct, table_sample)
- **iteration**: Iteration utilities (enum)

## Usage

```rust,ignore
use crate::builtins::call_builtin_function;

let result = call_builtin_function("table_where", args, line_number)?;
```
*/

// Module declarations
pub mod system;
pub mod file;
pub mod file_io;
pub mod registry;
pub mod math;
pub mod array;
pub mod string;
pub mod table;
pub mod filter;
pub mod iteration;

// Re-exports for convenience


use crate::value::Value;
use crate::error::{DataCodeError, Result};


/// Main entry point for all built-in function calls
///
/// This function routes function calls to the appropriate module based on the function name.
/// It provides a unified interface for all built-in functions while maintaining modular organization.
///
/// OPTIMIZED: Uses hash table registry for O(1) function lookup
pub fn call_builtin_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    // Try optimized registry first (O(1) lookup)
    if registry::function_exists(name) {
        return registry::call_builtin_function_fast(name, args, line);
    }
    // Route to appropriate module based on function name
    if system::is_system_function(name) {
        system::call_system_function(name, args, line)
    } else if file::is_file_function(name) {
        file::call_file_function(name, args, line)
    } else if math::is_math_function(name) {
        math::call_math_function(name, args, line)
    } else if array::is_array_function(name) {
        array::call_array_function(name, args, line)
    } else if string::is_string_function(name) {
        string::call_string_function(name, args, line)
    } else if table::is_table_function(name) {
        table::call_table_function(name, args, line)
    } else if filter::is_filter_function(name) {
        filter::call_filter_function(name, args, line)
    } else if iteration::is_iteration_function(name) {
        iteration::call_iteration_function(name, args, line)
    } else {
        Err(DataCodeError::function_not_found(name, line))
    }
}



/// Check if a function name is a valid built-in function
pub fn is_builtin_function(name: &str) -> bool {
    system::is_system_function(name) ||
    file::is_file_function(name) ||
    math::is_math_function(name) ||
    array::is_array_function(name) ||
    string::is_string_function(name) ||
    table::is_table_function(name) ||
    filter::is_filter_function(name) ||
    iteration::is_iteration_function(name)
}







#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_routing() {
        // Test that functions are correctly routed to their modules
        assert!(system::is_system_function("print"));
        assert!(math::is_math_function("abs"));
        assert!(array::is_array_function("push"));
        assert!(string::is_string_function("split"));
        assert!(table::is_table_function("table"));
        assert!(filter::is_filter_function("table_where"));
        assert!(iteration::is_iteration_function("enum"));
    }



    #[test]
    fn test_builtin_function_check() {
        assert!(is_builtin_function("print"));
        assert!(is_builtin_function("table_where"));
        assert!(is_builtin_function("abs"));
        assert!(!is_builtin_function("nonexistent_function"));
    }
}
