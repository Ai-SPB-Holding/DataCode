use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DataCodeError {
    // Синтаксические ошибки
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },
    
    // Ошибки времени выполнения
    RuntimeError {
        message: String,
        line: usize,
    },
    
    // Ошибки типов
    TypeError {
        expected: String,
        found: String,
        line: usize,
    },
    
    // Ошибки переменных
    VariableError {
        name: String,
        error_type: VariableErrorType,
        line: usize,
    },
    
    // Ошибки функций
    FunctionError {
        name: String,
        error_type: FunctionErrorType,
        line: usize,
    },
    
    // Ошибки файловой системы
    FileSystemError {
        path: String,
        error_type: FileSystemErrorType,
        line: usize,
    },
    
    // Ошибки парсинга выражений
    ExpressionError {
        expression: String,
        message: String,
        line: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableErrorType {
    NotFound,
    AlreadyDefined,
    InvalidScope,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionErrorType {
    NotFound,
    WrongArgumentCount { expected: usize, found: usize },
    InvalidArgument { index: usize, expected: String, found: String },
    InvalidReturn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileSystemErrorType {
    NotFound,
    PermissionDenied,
    InvalidPath,
    UnsupportedFormat,
    ReadError(String),
    WriteError(String),
}

impl fmt::Display for DataCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataCodeError::SyntaxError { message, line, column } => {
                write!(f, "Syntax Error at line {}, column {}: {}", line, column, message)
            }
            DataCodeError::RuntimeError { message, line } => {
                write!(f, "Runtime Error at line {}: {}", line, message)
            }
            DataCodeError::TypeError { expected, found, line } => {
                write!(f, "Type Error at line {}: expected {}, found {}", line, expected, found)
            }
            DataCodeError::VariableError { name, error_type, line } => {
                let msg = match error_type {
                    VariableErrorType::NotFound => format!("Variable '{}' not found", name),
                    VariableErrorType::AlreadyDefined => format!("Variable '{}' already defined", name),
                    VariableErrorType::InvalidScope => format!("Invalid scope for variable '{}'", name),
                };
                write!(f, "Variable Error at line {}: {}", line, msg)
            }
            DataCodeError::FunctionError { name, error_type, line } => {
                let msg = match error_type {
                    FunctionErrorType::NotFound => format!("Function '{}' not found", name),
                    FunctionErrorType::WrongArgumentCount { expected, found } => {
                        format!("Function '{}' expects {} arguments, found {}", name, expected, found)
                    }
                    FunctionErrorType::InvalidArgument { index, expected, found } => {
                        format!("Function '{}' argument {} expects {}, found {}", name, index + 1, expected, found)
                    }
                    FunctionErrorType::InvalidReturn => format!("Function '{}' returned invalid value", name),
                };
                write!(f, "Function Error at line {}: {}", line, msg)
            }
            DataCodeError::FileSystemError { path, error_type, line } => {
                let msg = match error_type {
                    FileSystemErrorType::NotFound => format!("File or directory '{}' not found", path),
                    FileSystemErrorType::PermissionDenied => format!("Permission denied for '{}'", path),
                    FileSystemErrorType::InvalidPath => format!("Invalid path '{}'", path),
                    FileSystemErrorType::UnsupportedFormat => format!("Unsupported file format for '{}'", path),
                    FileSystemErrorType::ReadError(e) => format!("Failed to read '{}': {}", path, e),
                    FileSystemErrorType::WriteError(e) => format!("Failed to write '{}': {}", path, e),
                };
                write!(f, "File System Error at line {}: {}", line, msg)
            }
            DataCodeError::ExpressionError { expression, message, line } => {
                write!(f, "Expression Error at line {}: {} in '{}'", line, message, expression)
            }
        }
    }
}

impl std::error::Error for DataCodeError {}

// Реализация From для автоматического преобразования ошибок
impl From<String> for DataCodeError {
    fn from(msg: String) -> Self {
        DataCodeError::RuntimeError {
            message: msg,
            line: 0, // Будет обновлено в контексте
        }
    }
}

impl From<&str> for DataCodeError {
    fn from(msg: &str) -> Self {
        DataCodeError::RuntimeError {
            message: msg.to_string(),
            line: 0, // Будет обновлено в контексте
        }
    }
}

pub type Result<T> = std::result::Result<T, DataCodeError>;

// Вспомогательные функции для создания ошибок
impl DataCodeError {
    pub fn syntax_error(message: &str, line: usize, column: usize) -> Self {
        DataCodeError::SyntaxError {
            message: message.to_string(),
            line,
            column,
        }
    }
    
    pub fn runtime_error(message: &str, line: usize) -> Self {
        DataCodeError::RuntimeError {
            message: message.to_string(),
            line,
        }
    }
    
    pub fn type_error(expected: &str, found: &str, line: usize) -> Self {
        DataCodeError::TypeError {
            expected: expected.to_string(),
            found: found.to_string(),
            line,
        }
    }
    
    pub fn variable_not_found(name: &str, line: usize) -> Self {
        DataCodeError::VariableError {
            name: name.to_string(),
            error_type: VariableErrorType::NotFound,
            line,
        }
    }
    
    pub fn function_not_found(name: &str, line: usize) -> Self {
        DataCodeError::FunctionError {
            name: name.to_string(),
            error_type: FunctionErrorType::NotFound,
            line,
        }
    }
    
    pub fn wrong_argument_count(name: &str, expected: usize, found: usize, line: usize) -> Self {
        DataCodeError::FunctionError {
            name: name.to_string(),
            error_type: FunctionErrorType::WrongArgumentCount { expected, found },
            line,
        }
    }
    
    pub fn file_not_found(path: &str, line: usize) -> Self {
        DataCodeError::FileSystemError {
            path: path.to_string(),
            error_type: FileSystemErrorType::NotFound,
            line,
        }
    }
    
    pub fn expression_error(expression: &str, message: &str, line: usize) -> Self {
        DataCodeError::ExpressionError {
            expression: expression.to_string(),
            message: message.to_string(),
            line,
        }
    }
}
