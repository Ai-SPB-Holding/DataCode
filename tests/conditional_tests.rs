use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[cfg(test)]
mod conditional_tests {
    use super::*;

    #[test]
    fn test_simple_if_condition() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 10").unwrap();
        interp.exec("global result = 'default'").unwrap();
        
        let if_code = r#"if x > 5 do
    global result = 'greater'
endif"#;
        
        interp.exec(if_code).unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::String("greater".to_string())));
    }

    #[test]
    fn test_if_condition_false() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 3").unwrap();
        interp.exec("global result = 'default'").unwrap();
        
        let if_code = r#"if x > 5 do
    global result = 'greater'
endif"#;
        
        interp.exec(if_code).unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::String("default".to_string())));
    }

    #[test]
    fn test_if_else_condition() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 3").unwrap();
        
        let if_code = r#"if x > 5 do
    global result = 'greater'
else
    global result = 'not_greater'
endif"#;
        
        interp.exec(if_code).unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::String("not_greater".to_string())));
    }

    #[test]
    fn test_if_else_condition_true() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 10").unwrap();
        
        let if_code = r#"if x > 5 do
    global result = 'greater'
else
    global result = 'not_greater'
endif"#;
        
        interp.exec(if_code).unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::String("greater".to_string())));
    }

    #[test]
    fn test_nested_if_conditions() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 15").unwrap();
        
        let if_code = r#"if x > 0 do
    if x > 10 do
        global result = 'big_positive'
    else
        global result = 'small_positive'
    endif
else
    global result = 'negative'
endif"#;
        
        interp.exec(if_code).unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::String("big_positive".to_string())));
    }

    #[test]
    fn test_nested_if_conditions_small() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 5").unwrap();
        
        let if_code = r#"if x > 0 do
    if x > 10 do
        global result = 'big_positive'
    else
        global result = 'small_positive'
    endif
else
    global result = 'negative'
endif"#;
        
        interp.exec(if_code).unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::String("small_positive".to_string())));
    }

    #[test]
    fn test_comparison_operators() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 10").unwrap();
        interp.exec("global y = 5").unwrap();
        
        // Тест >
        let if_code1 = r#"if x > y do
    global result1 = true
else
    global result1 = false
endif"#;
        interp.exec(if_code1).unwrap();
        assert_eq!(interp.get_variable("result1"), Some(&Value::Bool(true)));
        
        // Тест <
        let if_code2 = r#"if x < y do
    global result2 = true
else
    global result2 = false
endif"#;
        interp.exec(if_code2).unwrap();
        assert_eq!(interp.get_variable("result2"), Some(&Value::Bool(false)));
        
        // Тест ==
        let if_code3 = r#"if x == 10 do
    global result3 = true
else
    global result3 = false
endif"#;
        interp.exec(if_code3).unwrap();
        assert_eq!(interp.get_variable("result3"), Some(&Value::Bool(true)));
        
        // Тест !=
        let if_code4 = r#"if x != y do
    global result4 = true
else
    global result4 = false
endif"#;
        interp.exec(if_code4).unwrap();
        assert_eq!(interp.get_variable("result4"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_logical_operators_in_conditions() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 10").unwrap();
        interp.exec("global y = 5").unwrap();
        interp.exec("global flag = true").unwrap();
        
        // Тест AND
        let if_code1 = r#"if (x > y) and flag do
    global result1 = 'both_true'
else
    global result1 = 'not_both'
endif"#;
        interp.exec(if_code1).unwrap();
        assert_eq!(interp.get_variable("result1"), Some(&Value::String("both_true".to_string())));
        
        // Тест OR
        let if_code2 = r#"if (x < y) or flag do
    global result2 = 'at_least_one'
else
    global result2 = 'none'
endif"#;
        interp.exec(if_code2).unwrap();
        assert_eq!(interp.get_variable("result2"), Some(&Value::String("at_least_one".to_string())));
        
        // Тест NOT
        let if_code3 = r#"if not (x < y) do
    global result3 = 'not_less'
else
    global result3 = 'is_less'
endif"#;
        interp.exec(if_code3).unwrap();
        assert_eq!(interp.get_variable("result3"), Some(&Value::String("not_less".to_string())));
    }

    #[test]
    fn test_string_conditions() {
        let mut interp = Interpreter::new();
        
        interp.exec("global name = 'DataCode'").unwrap();
        interp.exec("global version = '1.0'").unwrap();
        
        let if_code1 = r#"if name == 'DataCode' do
    global result1 = 'correct_name'
else
    global result1 = 'wrong_name'
endif"#;
        interp.exec(if_code1).unwrap();
        assert_eq!(interp.get_variable("result1"), Some(&Value::String("correct_name".to_string())));
        
        let if_code2 = r#"if version != '2.0' do
    global result2 = 'not_v2'
else
    global result2 = 'is_v2'
endif"#;
        interp.exec(if_code2).unwrap();
        assert_eq!(interp.get_variable("result2"), Some(&Value::String("not_v2".to_string())));
    }

    #[test]
    fn test_boolean_conditions() {
        let mut interp = Interpreter::new();
        
        interp.exec("global flag1 = true").unwrap();
        interp.exec("global flag2 = false").unwrap();
        
        let if_code1 = r#"if flag1 do
    global result1 = 'flag1_true'
else
    global result1 = 'flag1_false'
endif"#;
        interp.exec(if_code1).unwrap();
        assert_eq!(interp.get_variable("result1"), Some(&Value::String("flag1_true".to_string())));
        
        let if_code2 = r#"if flag2 do
    global result2 = 'flag2_true'
else
    global result2 = 'flag2_false'
endif"#;
        interp.exec(if_code2).unwrap();
        assert_eq!(interp.get_variable("result2"), Some(&Value::String("flag2_false".to_string())));
    }

    #[test]
    fn test_if_with_function_calls() {
        let mut interp = Interpreter::new();
        
        // Определяем простую функцию БЕЗ условных конструкций
        let function_code = r#"global function is_positive(x) do
    return x > 0
endfunction"#;
        
        interp.exec(function_code).unwrap();
        
        // Используем функцию в условии
        let if_code = r#"if is_positive(10) do
    global result = 'positive'
else
    global result = 'not_positive'
endif"#;
        
        interp.exec(if_code).unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::String("positive".to_string())));
    }

    #[test]
    fn test_if_syntax_errors() {
        let mut interp = Interpreter::new();
        
        // Отсутствует endif
        let bad_if1 = r#"if x > 5 do
    global result = 'test'"#;
        let result = interp.exec(bad_if1);
        assert!(result.is_err());
        
        // Отсутствует do
        let bad_if2 = r#"if x > 5
    global result = 'test'
endif"#;
        let result = interp.exec(bad_if2);
        assert!(result.is_err());
        
        // Неправильный синтаксис условия
        let bad_if3 = r#"if do
    global result = 'test'
endif"#;
        let result = interp.exec(bad_if3);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_nested_conditions() {
        let mut interp = Interpreter::new();
        
        interp.exec("global score = 85").unwrap();
        
        let complex_if = r#"if score >= 90 do
    global grade = 'A'
else
    if score >= 80 do
        global grade = 'B'
    else
        if score >= 70 do
            global grade = 'C'
        else
            if score >= 60 do
                global grade = 'D'
            else
                global grade = 'F'
            endif
        endif
    endif
endif"#;
        
        interp.exec(complex_if).unwrap();
        assert_eq!(interp.get_variable("grade"), Some(&Value::String("B".to_string())));
    }
}
