use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;

#[test]
fn test_basic_variable_assignment() {
    let mut interp = Interpreter::new();
    let result = interp.exec("global greeting = 'hi'");
    assert!(result.is_ok());
    let val = interp.get_variable("greeting").unwrap();
    assert_eq!(val, &Value::String("hi".to_string()));
}

#[test]
fn test_path_building() {
    let mut interp = Interpreter::new();
    interp.set_variable("root".to_string(), Value::Path("/base".into()), true);

    // Проверяем, что переменная установлена правильно
    let val = interp.get_variable("root").unwrap();
    match val {
        Value::Path(p) => assert_eq!(p, &std::path::PathBuf::from("/base")),
        _ => panic!("Expected a Path"),
    }
}

#[test]
fn test_error_path_building() {
    let mut interp = Interpreter::new();
    interp.set_variable("root".to_string(), Value::Path("/base".into()), true);
    
    let result = interp.exec("global full = root / folder");
    
    match result.unwrap_err() {
        DataCodeError::VariableError { name, .. } => {
            assert_eq!(name, "folder");
        }
        _ => panic!("Expected VariableError"),
    }
}

#[test]
fn test_string_addition() {
    let mut interp = Interpreter::new();
    interp.exec("global s = 'foo' + 'bar'").unwrap();
    let val = interp.get_variable("s").unwrap();
    assert_eq!(val, &Value::String("foobar".to_string()));
}

#[test]
fn test_for_loop_accumulation() {
    let mut interp = Interpreter::new();
    interp.set_variable("items".to_string(), Value::Array(vec![
        Value::String("1".into()),
        Value::String("2".into()),
    ]), true);

    let code = "\
        for item in items do
            global last = item
        next item";

    interp.exec(code).unwrap();

    let last = interp.get_variable("last").unwrap();
    assert_eq!(last, &Value::String("2".into()));
}