/*!
# DataCode Built-in Functions Module

This module provides a modular architecture for DataCode's built-in functions,
organized by functionality for better maintainability and extensibility.

## Module Structure

- **system**: System utilities (now, print, getcwd, isinstance)
- **file**: File operations (path, list_files, read_file, analyze_csv)
- **math**: Mathematical functions (abs, sqrt, pow, min, max, round, div)
- **array**: Array operations (len, push, pop, sort, unique, sum, average)
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
pub mod sqlite_export;

// Re-exports for convenience


use crate::value::Value;
use crate::error::{DataCodeError, Result};
use std::collections::HashMap;


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

/// Call built-in function with named arguments support
/// This function converts named arguments to positional arguments for functions that support them
pub fn call_builtin_function_with_named_args(
    name: &str,
    args: Vec<Value>,
    named_args: HashMap<String, Value>,
    line: usize
) -> Result<Value> {
    // If no named arguments, use the regular function
    if named_args.is_empty() {
        return call_builtin_function(name, args, line);
    }

    // Handle functions that support named arguments
    match name {
        "read_file" => {
            // Convert named arguments to positional arguments for read_file
            let mut final_args = args;
            
            // Handle sheet_name named argument
            if let Some(sheet_name) = named_args.get("sheet_name") {
                if final_args.len() == 1 {
                    // read_file(path, sheet_name=...)
                    final_args.push(sheet_name.clone());
                } else if final_args.len() == 2 {
                    // Check if second arg is a number (header_row) or string (sheet_name)
                    // If it's a number, we have read_file(path, header_row, sheet_name=...)
                    // If it's a string, we replace it with the named argument
                    match &final_args[1] {
                        Value::Number(_) => {
                            // It's a header_row, add sheet_name as third argument
                            final_args.push(sheet_name.clone());
                        }
                        Value::String(_) => {
                            // It's already a sheet_name, replace it with named argument
                            final_args[1] = sheet_name.clone();
                        }
                        _ => {
                            return Err(DataCodeError::runtime_error(
                                "Invalid argument type for read_file",
                                line
                            ));
                        }
                    }
                } else if final_args.len() == 3 {
                    // read_file(path, header_row, sheet_name) - replace third arg
                    final_args[2] = sheet_name.clone();
                }
            }
            
            // Handle header_row named argument
            if let Some(header_row) = named_args.get("header_row") {
                match header_row {
                    Value::Number(_n) => {
                        if final_args.len() == 1 {
                            // read_file(path, header_row=...)
                            final_args.push(header_row.clone());
                        } else if final_args.len() == 2 {
                            // Check if second arg is a string (sheet_name)
                            if let Value::String(_) = &final_args[1] {
                                // We have sheet_name, insert header_row before it
                                let sheet_name = final_args.remove(1);
                                final_args.push(header_row.clone());
                                final_args.push(sheet_name);
                            } else {
                                // Replace second arg
                                final_args[1] = header_row.clone();
                            }
                        } else if final_args.len() == 3 {
                            // Replace second arg (header_row)
                            final_args[1] = header_row.clone();
                        }
                    }
                    _ => {
                        return Err(DataCodeError::runtime_error(
                            "header_row must be a number",
                            line
                        ));
                    }
                }
            }
            
            // Handle header named argument - pass it directly to read_file
            // We'll handle it in call_file_function by passing named_args
            let header_arg = named_args.get("header").cloned();
            
            // Check for unknown named arguments
            for (key, _) in &named_args {
                if key != "sheet_name" && key != "header_row" && key != "header" {
                    return Err(DataCodeError::runtime_error(
                        &format!("Unknown named argument '{}' for read_file", key),
                        line
                    ));
                }
            }
            
            // Call read_file with header through file module
            crate::builtins::file::call_file_function_with_header("read_file", final_args, header_arg, line)
        }
        _ => {
            // For other functions, check if they have named arguments (not supported yet)
            if !named_args.is_empty() {
                return Err(DataCodeError::runtime_error(
                    &format!("Function '{}' does not support named arguments", name),
                    line
                ));
            }
            call_builtin_function(name, args, line)
        }
    }
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
