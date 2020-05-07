use forth::ForthInterp;
use std::io;

mod forth;

fn slurp_expr() -> String {
    let mut expr = String::new();

    io::stdin()
        .read_line(&mut expr)
        .expect("Failed to read line");

    expr
}

fn main() {
    let mut interp = ForthInterp::new();
    loop {
        println!("rforth >");
        let expr = slurp_expr();
        match interp.eval_str(&expr) {
            Ok(_) => {
                print!("// stack => ");
                for exp in &interp.stack {
                    print!("{} ", exp)
                }
                println!();
            }

            Err(e) => println!("// err => {}", e),
        }
    }
}
