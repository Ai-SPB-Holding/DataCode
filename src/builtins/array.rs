use crate::value::Value;
use crate::error::{DataCodeError, Result};

/// Array operations functions
pub fn call_array_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "length" | "len" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count(name, 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => Ok(Number(arr.len() as f64)),
                String(s) => Ok(Number(s.len() as f64)),
                Currency(c) => Ok(Number(c.len() as f64)),
                Table(table) => Ok(Number(table.borrow().rows.len() as f64)),
                _ => Err(DataCodeError::type_error("Array, String, Currency, or Table", "other", line)),
            }
        }
        
        "push" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("push", 2, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    // Phase 1 Aggressive Optimization: Use Vec::with_capacity for better performance
                    // For very large arrays, use a more aggressive growth strategy
                    let new_capacity = if arr.is_empty() {
                        16  // Start with larger initial capacity
                    } else if arr.len() < 1000 {
                        (arr.len() * 2).max(arr.len() + 100)  // Aggressive growth for small-medium arrays
                    } else if arr.len() < 10000 {
                        arr.len() + (arr.len() / 2)  // 1.5x growth for large arrays
                    } else {
                        arr.len() + (arr.len() / 4)  // 1.25x growth for very large arrays
                    };

                    let mut new_arr = Vec::with_capacity(new_capacity);
                    new_arr.extend_from_slice(arr);
                    new_arr.push(args[1].clone());
                    Ok(Array(new_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }

        "array_builder" => {
            // Phase 1 Optimization: Create an array builder for efficient bulk operations
            if args.len() > 1 {
                return Err(DataCodeError::wrong_argument_count("array_builder", 0, args.len(), line));
            }
            let initial_capacity = if args.is_empty() { 1000 } else {
                match &args[0] {
                    Number(n) => *n as usize,
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            };
            Ok(Array(Vec::with_capacity(initial_capacity)))
        }

        "extend" => {
            // Phase 1 Optimization: Bulk extend operation
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("extend", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Array(arr1), Array(arr2)) => {
                    let mut new_arr = Vec::with_capacity(arr1.len() + arr2.len());
                    new_arr.extend_from_slice(arr1);
                    new_arr.extend_from_slice(arr2);
                    Ok(Array(new_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }

        "bulk_create" => {
            // Phase 1 Aggressive Optimization: Create array in bulk to avoid O(n²) complexity
            if args.len() != 3 {
                return Err(DataCodeError::wrong_argument_count("bulk_create", 3, args.len(), line));
            }
            match (&args[0], &args[1], &args[2]) {
                (Number(count), Array(template), Array(params)) => {
                    let count = *count as usize;
                    let mut result = Vec::with_capacity(count);

                    for i in 0..count {
                        // Create row based on template and parameters
                        let mut row = Vec::with_capacity(template.len());
                        for (j, template_item) in template.iter().enumerate() {
                            match template_item {
                                String(s) if s == "INDEX" => {
                                    row.push(Number(i as f64));
                                }
                                String(s) if s.starts_with("PARAM_") => {
                                    if let Ok(param_idx) = s[6..].parse::<usize>() {
                                        if param_idx < params.len() {
                                            row.push(params[param_idx].clone());
                                        } else {
                                            row.push(Value::Null);
                                        }
                                    } else {
                                        row.push(template_item.clone());
                                    }
                                }
                                _ => row.push(template_item.clone()),
                            }
                        }
                        result.push(Array(row));
                    }

                    Ok(Array(result))
                }
                _ => Err(DataCodeError::type_error("Number, Array, Array", "other", line)),
            }
        }
        
        "pop" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("pop", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    if arr.is_empty() {
                        Ok(Null)
                    } else {
                        Ok(arr[arr.len() - 1].clone())
                    }
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "append" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("append", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Array(arr), value) => {
                    let mut new_arr = arr.clone();
                    new_arr.push(value.clone());
                    Ok(Array(new_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "sort" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("sort", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut sorted_arr = arr.clone();
                    sorted_arr.sort_by(|a, b| {
                        match (a, b) {
                            (Number(n1), Number(n2)) => n1.partial_cmp(n2).unwrap_or(std::cmp::Ordering::Equal),
                            (String(s1), String(s2)) => s1.cmp(s2),
                            _ => std::cmp::Ordering::Equal,
                        }
                    });
                    Ok(Array(sorted_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "unique" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("unique", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut unique_items = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    
                    for item in arr {
                        let key = format!("{:?}", item);
                        if seen.insert(key) {
                            unique_items.push(item.clone());
                        }
                    }
                    
                    Ok(Array(unique_items))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "array" => {
            if !args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("array", 0, args.len(), line));
            }
            Ok(Array(vec![]))
        }
        
        "sum" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("sum", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut total = 0.0;
                    for item in arr {
                        match item {
                            Number(n) => total += n,
                            _ => return Err(DataCodeError::type_error("Array of Numbers", "other", line)),
                        }
                    }
                    Ok(Number(total))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "average" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("average", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    if arr.is_empty() {
                        return Err(DataCodeError::runtime_error("Cannot calculate average of empty array", line));
                    }
                    let mut total = 0.0;
                    for item in arr {
                        match item {
                            Number(n) => total += n,
                            _ => return Err(DataCodeError::type_error("Array of Numbers", "other", line)),
                        }
                    }
                    Ok(Number(total / arr.len() as f64))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "count" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("count", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => Ok(Number(arr.len() as f64)),
                String(s) => Ok(Number(s.len() as f64)),
                Table(table) => Ok(Number(table.borrow().rows.len() as f64)),
                _ => Err(DataCodeError::type_error("Array, String, or Table", "other", line)),
            }
        }

        "range" => {
            match args.len() {
                1 => {
                    // range(n) -> [0, 1, 2, ..., n-1]
                    let end = match &args[0] {
                        Number(n) => *n as i32,
                        _ => return Err(DataCodeError::type_error("Number", "other", line)),
                    };

                    if end < 0 {
                        return Err(DataCodeError::runtime_error("Range end cannot be negative", line));
                    }

                    let range: Vec<Value> = (0..end)
                        .map(|i| Number(i as f64))
                        .collect();
                    Ok(Array(range))
                }
                2 => {
                    // range(start, end) -> [start, start+1, ..., end-1]
                    let start = match &args[0] {
                        Number(n) => *n as i32,
                        _ => return Err(DataCodeError::type_error("Number", "other", line)),
                    };
                    let end = match &args[1] {
                        Number(n) => *n as i32,
                        _ => return Err(DataCodeError::type_error("Number", "other", line)),
                    };

                    let range: Vec<Value> = (start..end)
                        .map(|i| Number(i as f64))
                        .collect();
                    Ok(Array(range))
                }
                3 => {
                    // range(start, end, step)
                    let start = match &args[0] {
                        Number(n) => *n as i32,
                        _ => return Err(DataCodeError::type_error("Number", "other", line)),
                    };
                    let end = match &args[1] {
                        Number(n) => *n as i32,
                        _ => return Err(DataCodeError::type_error("Number", "other", line)),
                    };
                    let step = match &args[2] {
                        Number(n) => *n as i32,
                        _ => return Err(DataCodeError::type_error("Number", "other", line)),
                    };

                    if step == 0 {
                        return Err(DataCodeError::runtime_error("Range step cannot be zero", line));
                    }

                    let mut range = Vec::new();
                    if step > 0 {
                        let mut current = start;
                        while current < end {
                            range.push(Number(current as f64));
                            current += step;
                        }
                    } else {
                        let mut current = start;
                        while current > end {
                            range.push(Number(current as f64));
                            current += step;
                        }
                    }

                    Ok(Array(range))
                }
                _ => Err(DataCodeError::wrong_argument_count("range", 1, args.len(), line))
            }
        }

        "map" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("map", 2, args.len(), line));
            }

            match (&args[0], &args[1]) {
                (Array(arr), String(func_name)) => {
                    let mut result = Vec::new();

                    for item in arr {
                        // Вызываем функцию для каждого элемента
                        let mapped_value = call_function_with_single_arg(func_name, item.clone(), line)?;
                        result.push(mapped_value);
                    }

                    Ok(Array(result))
                }
                _ => Err(DataCodeError::type_error("Array and String (function name)", "other", line)),
            }
        }

        "filter" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("filter", 2, args.len(), line));
            }

            match (&args[0], &args[1]) {
                (Array(arr), String(func_name)) => {
                    let mut result = Vec::new();

                    for item in arr {
                        // Вызываем функцию-предикат для каждого элемента
                        let should_include = call_function_with_single_arg(func_name, item.clone(), line)?;

                        // Проверяем результат как булево значение
                        let include = match should_include {
                            Value::Bool(b) => b,
                            Value::Number(n) => n != 0.0,
                            Value::String(s) => !s.is_empty(),
                            Value::Null => false,
                            _ => true,
                        };

                        if include {
                            result.push(item.clone());
                        }
                    }

                    Ok(Array(result))
                }
                _ => Err(DataCodeError::type_error("Array and String (function name)", "other", line)),
            }
        }

        "reduce" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(DataCodeError::wrong_argument_count("reduce", 2, args.len(), line));
            }

            match &args[0] {
                Array(arr) => {
                    if arr.is_empty() {
                        return if args.len() == 3 {
                            Ok(args[2].clone()) // Возвращаем начальное значение
                        } else {
                            Err(DataCodeError::runtime_error("Cannot reduce empty array without initial value", line))
                        };
                    }

                    let func_name = match &args[1] {
                        String(name) => name,
                        _ => return Err(DataCodeError::type_error("String (function name)", "other", line)),
                    };

                    // Определяем начальное значение и начальный индекс
                    let (mut accumulator, start_index) = if args.len() == 3 {
                        (args[2].clone(), 0)
                    } else {
                        (arr[0].clone(), 1)
                    };

                    // Применяем функцию к каждому элементу
                    for i in start_index..arr.len() {
                        accumulator = call_function_with_two_args(func_name, accumulator, arr[i].clone(), line)?;
                    }

                    Ok(accumulator)
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }

        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to array functions
pub fn is_array_function(name: &str) -> bool {
    matches!(name,
        "length" | "len" | "push" | "pop" | "append" | "sort" |
        "unique" | "array" | "sum" | "average" | "count" | "range" |
        "map" | "filter" | "reduce" | "array_builder" | "extend" | "bulk_create"
    )
}

/// Вызвать функцию с одним аргументом (для map и filter)
fn call_function_with_single_arg(func_name: &str, arg: Value, line: usize) -> Result<Value> {
    // Сначала проверяем встроенные функции
    if crate::builtins::is_builtin_function(func_name) {
        return crate::builtins::call_builtin_function(func_name, vec![arg], line);
    }

    // Для пользовательских функций нужен доступ к интерпретатору
    // Пока что возвращаем ошибку - это будет исправлено в следующих версиях
    Err(DataCodeError::runtime_error(
        &format!("User function '{}' calls in functional methods not yet supported", func_name),
        line,
    ))
}

/// Вызвать функцию с двумя аргументами (для reduce)
fn call_function_with_two_args(func_name: &str, arg1: Value, arg2: Value, line: usize) -> Result<Value> {
    // Сначала проверяем встроенные функции
    if crate::builtins::is_builtin_function(func_name) {
        return crate::builtins::call_builtin_function(func_name, vec![arg1, arg2], line);
    }

    // Для пользовательских функций нужен доступ к интерпретатору
    // Пока что возвращаем ошибку - это будет исправлено в следующих версиях
    Err(DataCodeError::runtime_error(
        &format!("User function '{}' calls in functional methods not yet supported", func_name),
        line,
    ))
}
