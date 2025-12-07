# 🛠️ DataCode Developer Tools

This section demonstrates tools for development, debugging, and performance testing in DataCode.

## 📋 Contents

### 1. `debug_mode_test.dc` - Debug Mode Testing
**Description**: Demonstrates debugging and diagnostics capabilities in DataCode.

**What you'll learn**:
- Debug mode
- Diagnostic messages
- Execution tracing
- Debug information

**How to use**:
```bash
# Run in normal mode
cargo run examples/06-developer-tools/debug_mode_test.dc

# Run with debug information (if supported)
cargo run -- --debug examples/06-developer-tools/debug_mode_test.dc
```

### 2. `interactive_demo.dc` - Interactive Demonstrations
**Description**: Code examples intended for execution in interactive mode (REPL).

**What you'll learn**:
- Working in REPL mode
- Interactive testing
- Step-by-step execution
- Code experimentation

**How to use**:
```bash
# Start interactive mode
cargo run

# Then copy and paste code from file
DataCode> # code from interactive_demo.dc
```

### 3. `stress_benchmark.dc` - Performance Testing
**Description**: Benchmarks for testing performance of various DataCode operations.

**What you'll learn**:
- Performance measurement
- Stress testing
- Code optimization
- Bottleneck analysis

**Warning**: ⚠️ This file may take long to execute and consume many resources!

## 🎯 How to Run Examples

```bash
# Debug testing
cargo run examples/06-developer-tools/debug_mode_test.dc

# Interactive examples (start REPL)
cargo run
# Then use code from interactive_demo.dc

# Benchmarks (be careful!)
cargo run examples/06-developer-tools/stress_benchmark.dc
```

## 🔧 Debugging Tools

### Debug Mode
```datacode
# Add debug print() for tracing
global function debug_function(x) do
    print('DEBUG: Input value:', x)
    local result = x * 2
    print('DEBUG: Result:', result)
    return result
endfunction
```

### State Checking
```datacode
# Check variables at critical points
global x = 10
print('State of x:', x)
if x > 5 do
    print('DEBUG: x is greater than 5')
endif
```

### Execution Tracing
```datacode
global function traced_factorial(n) do
    print('TRACE: factorial(', n, ') called')
    if n <= 1 do
        print('TRACE: base case, returning 1')
        return 1
    endif
    local result = n * traced_factorial(n - 1)
    print('TRACE: factorial(', n, ') = ', result)
    return result
endfunction
```

## 📊 Performance Testing

### Measuring Execution Time
```datacode
# Use now() function to measure time
global start_time = now()

# Your code to test
for i in [1, 2, 3, 4, 5] do
    # some operations
next i

global end_time = now()
print('Execution time:', end_time - start_time)
```

### Stress Tests
```datacode
# Testing with large data
global large_array = []
for i in range(1000) do
    push(large_array, i)
next i

# Testing operation performance
global sum_result = sum(large_array)
```

## 🔍 Interactive Development

### REPL Mode
```bash
cargo run
DataCode> global x = 10
DataCode> print(x)
10
DataCode> global function test() do return x * 2 endfunction
DataCode> print(test())
20
```

### Step-by-step Testing
1. Start REPL: `cargo run`
2. Define variables: `global x = 5`
3. Test functions: `print(x + 10)`
4. Experiment with code
5. Use command history (up/down arrows)

## ⚠️ Important Features

### Debugging
- Use `print()` to output debug information
- Add `DEBUG:` or `TRACE:` prefixes for clarity
- Check variable state at critical points

### Performance
- `stress_benchmark.dc` may take very long to execute
- Monitor memory consumption with large data
- Use `now()` to measure time

### Interactive Mode
- REPL supports command history
- Can define functions and variables step-by-step
- Use for quick testing of ideas

## 💡 Practical Tips

1. **Debugging**: Add temporary print() to understand execution flow
2. **REPL**: Use for quick testing of small code fragments
3. **Benchmarks**: Run on small data first
4. **Profiling**: Measure time of critical operations
5. **Iterative development**: Test code in parts

## 🔗 Navigation

### Previous Sections
- **[01-basics](../01-basics/)** - basic concepts
- **[02-language-syntax](../02-language-syntax/)** - syntax constructs
- **[03-advanced-features](../03-advanced-features/)** - error handling and debugging
- **[04-data-processing](../04-data-processing/)** - data processing optimization
- **[05-data-types](../05-data-types/)** - type system

### Final Section
- **[07-demonstrations](../07-demonstrations/)** - comprehensive examples and showcase

### Additional Resources
- **[../INDEX.md](../INDEX.md)** - 📋 Quick index of all examples
- **[../README.md](../README.md)** - 📚 Main examples page

## 📈 Recommended Learning Order

1. **`interactive_demo.dc`** - learn REPL mode
2. **`debug_mode_test.dc`** - master debugging
3. **`stress_benchmark.dc`** - performance testing (careful!)

## ⚡ Warnings

- **Benchmarks may be slow** - start with small data
- **Debug output** may slow execution
- **REPL mode** - best way to learn the language
- **Performance measurement** is important for optimization

---

**The right tools make DataCode development efficient!** 🛠️✨
