/*!
# DataCode Built-in Functions (Legacy)

This file provides backward compatibility for the old builtins interface.
New code should use the `builtins` module instead.

The modular structure provides better organization and maintainability:
- `builtins::system` - System utilities
- `builtins::file` - File operations  
- `builtins::math` - Mathematical functions
- `builtins::array` - Array operations
- `builtins::string` - String manipulation
- `builtins::table` - Table operations
- `builtins::filter` - Table filtering
- `builtins::iteration` - Iteration utilities
*/

// Re-export the modular builtins for backward compatibility
pub use crate::builtins::call_builtin_function as call_function;
