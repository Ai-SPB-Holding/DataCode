use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[test]
fn test_basic_variable_assignment() {
    let mut interp = Interpreter::new();
    interp.exec("global greeting = 'hi'").unwrap();
    let val = interp.get_variable("greeting").unwrap();
    assert_eq!(val, &Value::String("hi".to_string()));
}

#[test]
fn test_path_building() {
    let mut interp = Interpreter::new();
    interp.set_variable("root", Value::Path("/base".into()));
    interp.exec("global full = root / 'folder'").unwrap();
    let val = interp.get_variable("full").unwrap();
    match val {
        Value::Path(p) => assert_eq!(p, &std::path::PathBuf::from("/base/folder")),
        _ => panic!("Expected a Path"),
    }
}

#[test]
fn test_error_path_building() {
    let mut interp = Interpreter::new();
    interp.set_variable("root", Value::Path("/base".into()));
    
    let result = interp.exec("global full = root / folder");
    
    assert_eq!(
        result.unwrap_err(),
        "Unsupported expression: folder"
    );
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
    interp.set_variable("items", Value::Array(vec![
        Value::String("1".into()),
        Value::String("2".into()),
    ]));

    let code = "\
        for item in items do
            global last = item
        forend";

    interp.exec(code).unwrap();

    let last = interp.get_variable("last").unwrap();
    assert_eq!(last, &Value::String("2".into()));
}