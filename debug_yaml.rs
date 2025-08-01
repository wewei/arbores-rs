use arbores::interpreter::parser::parse_from_string;

fn main() {
    let input = "(1 2 3)";
    let output = parse_from_string(input);
    match output.result {
        Ok(expressions) => {
            if let Some(expr) = expressions.first() {
                let pretty = expr.to_pretty_string();
                println!("Pretty string:");
                println!("{:?}", pretty);
                println!("Bytes:");
                println!("{:?}", pretty.as_bytes());
                println!("Lines:");
                for (i, line) in pretty.lines().enumerate() {
                    println!("  {}: {:?}", i, line);
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
