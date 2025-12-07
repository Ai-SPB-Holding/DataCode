use data_code::parser::{Parser, Token, Expr};

#[test]
fn test_multiline_array_parsing() {
    let input = r#"[
        1, 2, 3,
        4, 5, 6
    ]"#;
    
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression().unwrap();
    
    match expr {
        Expr::ArrayLiteral { elements } => {
            assert_eq!(elements.len(), 6);
        }
        _ => panic!("Expected array literal"),
    }
}

#[test]
fn test_multiline_function_call_parsing() {
    let input = r#"table_create([
        [1, 'Alice'],
        [2, 'Bob']
    ], ['id', 'name'])"#;
    
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression().unwrap();
    
    match expr {
        Expr::FunctionCall { name, args, named_args: _ } => {
            assert_eq!(name, "table_create");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_nested_multiline_arrays() {
    let input = r#"[
        [1, 2],
        [
            3, 4,
            5, 6
        ]
    ]"#;
    
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression().unwrap();
    
    match expr {
        Expr::ArrayLiteral { elements } => {
            assert_eq!(elements.len(), 2);
            // Both elements should be arrays
            for element in elements {
                match element {
                    Expr::ArrayLiteral { .. } => {},
                    _ => panic!("Expected nested array literal"),
                }
            }
        }
        _ => panic!("Expected array literal"),
    }
}

#[test]
fn test_trailing_comma_multiline() {
    let input = r#"[
        1, 2, 3,
        4, 5, 6,
    ]"#;
    
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression().unwrap();
    
    match expr {
        Expr::ArrayLiteral { elements } => {
            assert_eq!(elements.len(), 6);
        }
        _ => panic!("Expected array literal"),
    }
}

#[test]
fn test_multiline_parenthesized_expression() {
    let input = r#"(
        1 + 2 +
        3 + 4
    )"#;
    
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression().unwrap();
    
    match expr {
        Expr::Binary { .. } => {},
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_lexer_newline_handling() {
    let input = "[\n1,\n2,\n3\n]";
    
    let mut lexer = data_code::parser::Lexer::new(input);
    
    assert_eq!(lexer.next_token(), Token::LeftBracket);
    assert_eq!(lexer.next_token(), Token::Newline);
    assert_eq!(lexer.next_token(), Token::Number(1.0));
    assert_eq!(lexer.next_token(), Token::Comma);
    assert_eq!(lexer.next_token(), Token::Newline);
    assert_eq!(lexer.next_token(), Token::Number(2.0));
    assert_eq!(lexer.next_token(), Token::Comma);
    assert_eq!(lexer.next_token(), Token::Newline);
    assert_eq!(lexer.next_token(), Token::Number(3.0));
    assert_eq!(lexer.next_token(), Token::Newline);
    assert_eq!(lexer.next_token(), Token::RightBracket);
    assert_eq!(lexer.next_token(), Token::EOF);
}

#[test]
fn test_parser_skip_newlines() {
    let input = "\n\n\n42\n\n";
    
    let mut parser = Parser::new(input);
    parser.skip_newlines();
    
    match parser.current_token() {
        Token::Number(n) => assert_eq!(*n, 42.0),
        _ => panic!("Expected number token after skipping newlines"),
    }
}

#[test]
fn test_complex_multiline_structure() {
    let input = r#"table_create(
        [
            [1, 'Alice', [
                'Engineering',
                'Senior'
            ]],
            [2, 'Bob', [
                'Marketing',
                'Junior'
            ]]
        ],
        ['id', 'name', 'details']
    )"#;
    
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression().unwrap();
    
    match expr {
        Expr::FunctionCall { name, args, named_args: _ } => {
            assert_eq!(name, "table_create");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected function call"),
    }
}
