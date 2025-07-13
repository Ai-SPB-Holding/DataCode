use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[test]
fn test_single_line_comments() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
# Это однострочный комментарий
global x = 42
# Еще один комментарий
global y = 24
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что переменные были созданы
    assert_eq!(interpreter.variables.get("x"), Some(&Value::Number(42.0)));
    assert_eq!(interpreter.variables.get("y"), Some(&Value::Number(24.0)));
}

#[test]
fn test_multiline_comments() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
"""
Это многострочный комментарий
который может содержать любой текст
включая код:
global ignored = 999
"""
global x = 42
"""
Еще один блочный комментарий
в середине кода
"""
global y = 24
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что только нужные переменные были созданы
    assert_eq!(interpreter.variables.get("x"), Some(&Value::Number(42.0)));
    assert_eq!(interpreter.variables.get("y"), Some(&Value::Number(24.0)));
    assert!(!interpreter.variables.contains_key("ignored"));
}

#[test]
fn test_single_line_block_comments() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
global x = 42
""" Это однострочный блочный комментарий """
global y = 24
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что переменные были созданы
    assert_eq!(interpreter.variables.get("x"), Some(&Value::Number(42.0)));
    assert_eq!(interpreter.variables.get("y"), Some(&Value::Number(24.0)));
}

#[test]
fn test_mixed_comments() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
# Однострочный комментарий
global x = 10
"""
Многострочный комментарий
с несколькими строками
"""
global y = 20
""" Однострочный блочный """
# Еще один однострочный
global z = 30
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что все переменные были созданы
    assert_eq!(interpreter.variables.get("x"), Some(&Value::Number(10.0)));
    assert_eq!(interpreter.variables.get("y"), Some(&Value::Number(20.0)));
    assert_eq!(interpreter.variables.get("z"), Some(&Value::Number(30.0)));
}

#[test]
fn test_comments_with_code_constructs() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
global x = 5
"""
Этот комментарий содержит код, который не должен выполняться:
if x > 0 do
    global should_not_exist = 999
endif
for i in [1, 2, 3] do
    global also_should_not_exist = i
forend
"""
if x > 0 do
    global should_exist = x * 2
endif
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что только нужные переменные были созданы
    assert_eq!(interpreter.variables.get("x"), Some(&Value::Number(5.0)));
    assert_eq!(interpreter.variables.get("should_exist"), Some(&Value::Number(10.0)));
    assert!(!interpreter.variables.contains_key("should_not_exist"));
    assert!(!interpreter.variables.contains_key("also_should_not_exist"));
}
