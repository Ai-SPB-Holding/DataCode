// Performance tests for DataCode interpreter
// These tests execute performance-intensive .dc files and verify successful completion

#[cfg(test)]
mod tests {
    use data_code::run;
    use std::time::Instant;

    // Helper function to execute a test and measure time
    fn run_performance_test(source: &str, test_name: &str) -> Result<(), data_code::LangError> {
        let start = Instant::now();
        run(source)?;
        let duration = start.elapsed();
        println!("{} completed in {:?}", test_name, duration);
        Ok(())
    }

    #[test]
    fn test_simple_performance() {
        let source = include_str!("performance_tests/simple_performance_test.dc");
        run_performance_test(source, "Simple performance test")
            .expect("Simple performance test should complete without errors");
    }

    #[test]
    fn test_recursive_performance() {
        let source = include_str!("performance_tests/recursive_performance_test.dc");
        run_performance_test(source, "Recursive performance test")
            .expect("Recursive performance test should complete without errors");
    }

    #[test]
    fn test_memory_intensive() {
        let source = include_str!("performance_tests/memory_intensive_test.dc");
        run_performance_test(source, "Memory-intensive test")
            .expect("Memory-intensive test should complete without errors");
    }

    #[test]
    fn test_large_dataset() {
        let source = include_str!("performance_tests/large_dataset_test.dc");
        run_performance_test(source, "Large dataset test")
            .expect("Large dataset test should complete without errors");
    }
}

