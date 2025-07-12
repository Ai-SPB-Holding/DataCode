// backend/DataCode/src/main.rs
mod value;
mod builtins;
mod interpreter;

use interpreter::Interpreter;
use value::Value;

fn main() {
    let mut interp = Interpreter::new();

    // Пример кода DataCode:
    let code1 = "global basePath = getcwd()";
    let code2 = "local SearchPath = basePath / '/data/'";
    let code3 = "global files = list_files(basePath / '/data/')";
    let code4 = "print(SearchPath / file)";

    // Выполнить присваивание пути
    interp.exec(code1).expect("Failed exec code1");
    println!("basePath: {:?}", interp.get_variable("basePath"));

    interp.exec(code2).expect("Failed exec code2");
    println!("SearchPath: {:?}", interp.get_variable("SearchPath"));

    // Выполнить list_files
    interp.exec(code3).expect("Failed exec code3");
    interp.exec(code4).expect("Failed exec code4");
    // interp.exec(code5).expect("Failed exec code5");

    // Получить список файлов
    if let Some(files) = interp.get_variable("files") {
        println!("Files: {:?}", files);
    } else {
        println!("No files variable");
    }

    // Вызов now()
    let now = builtins::call_function("now", vec![]).unwrap();
    println!("Now: {:?}", now);
    println!("basePath: {:?}", interp.get_variable("basePath"));

    let code6 = "for file in files do 
                    print('-', SearchPath / file)
                    local a = read_file(SearchPath / file)
                    print(a)
                forend";
    interp.exec(code6).expect("Failed exec code6");
}
