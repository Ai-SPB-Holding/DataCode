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
pub mod math;
pub mod array;
pub mod string;
pub mod table;
pub mod filter;
pub mod iteration;

// Re-exports for convenience
pub use system::*;
pub use file::*;
pub use math::*;
pub use array::*;
pub use string::*;
pub use table::*;
pub use filter::*;
pub use iteration::*;

use crate::value::Value;
use crate::error::{DataCodeError, Result};

/// Main entry point for all built-in function calls
/// 
/// This function routes function calls to the appropriate module based on the function name.
/// It provides a unified interface for all built-in functions while maintaining modular organization.
pub fn call_builtin_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
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

/// Get a list of all available built-in functions
/// 
/// Returns a vector of function names organized by category.
/// Useful for documentation, auto-completion, and debugging.
pub fn get_all_builtin_functions() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("System", vec!["now", "print", "getcwd", "isinstance"]),
        ("File", vec!["path", "list_files", "read_file", "analyze_csv", "read_csv_safe"]),
        ("Math", vec!["abs", "sqrt", "pow", "min", "max", "round", "div"]),
        ("Array", vec!["length", "len", "push", "pop", "append", "sort", "unique", "array", "sum", "average", "count"]),
        ("String", vec!["split", "join", "trim", "upper", "lower", "contains"]),
        ("Table", vec!["table", "table_create", "show_table", "table_info", "table_head", "table_tail", "table_headers", "table_select", "table_sort"]),
        ("Filter", vec!["table_filter", "table_where", "table_query", "table_distinct", "table_sample", "table_between", "table_in", "table_is_null", "table_not_null"]),
        ("Iteration", vec!["enum"]),
    ]
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

/// Get function category for a given function name
pub fn get_function_category(name: &str) -> Option<&'static str> {
    if system::is_system_function(name) {
        Some("System")
    } else if file::is_file_function(name) {
        Some("File")
    } else if math::is_math_function(name) {
        Some("Math")
    } else if array::is_array_function(name) {
        Some("Array")
    } else if string::is_string_function(name) {
        Some("String")
    } else if table::is_table_function(name) {
        Some("Table")
    } else if filter::is_filter_function(name) {
        Some("Filter")
    } else if iteration::is_iteration_function(name) {
        Some("Iteration")
    } else {
        None
    }
}

/// Print all available functions organized by category
pub fn print_function_help() {
    println!("📚 DataCode Built-in Functions");
    println!("==============================");
    
    for (category, functions) in get_all_builtin_functions() {
        println!("\n🔧 {} Functions:", category);
        for func in functions {
            println!("  • {}", func);
        }
    }
    
    println!("\n💡 Total: {} functions across {} categories", 
        get_all_builtin_functions().iter().map(|(_, funcs)| funcs.len()).sum::<usize>(),
        get_all_builtin_functions().len()
    );
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
    fn test_function_categories() {
        assert_eq!(get_function_category("print"), Some("System"));
        assert_eq!(get_function_category("abs"), Some("Math"));
        assert_eq!(get_function_category("push"), Some("Array"));
        assert_eq!(get_function_category("split"), Some("String"));
        assert_eq!(get_function_category("table"), Some("Table"));
        assert_eq!(get_function_category("table_where"), Some("Filter"));
        assert_eq!(get_function_category("enum"), Some("Iteration"));
        assert_eq!(get_function_category("nonexistent"), None);
    }

    #[test]
    fn test_builtin_function_check() {
        assert!(is_builtin_function("print"));
        assert!(is_builtin_function("table_where"));
        assert!(is_builtin_function("abs"));
        assert!(!is_builtin_function("nonexistent_function"));
    }
}
