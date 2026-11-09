use lalrpop_util::lalrpop_mod;
use std::io::{self, Write};

lalrpop_mod!(pub calc); // incluye el parser generado desde src/calc.lalrpop

fn main() {
    let parser = calc::ExprParser::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut linea = String::new();
        if io::stdin().read_line(&mut linea).unwrap() == 0 {
            break; // fin de la entrada (Ctrl+D)
        }
        let linea = linea.trim();
        if linea == "q" {
            break;
        }

        match parser.parse(linea) {
            Ok(valor) => println!("= {}", valor),
            Err(e) => println!("error: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::calc::ExprParser;

    #[test]
    fn precedencia() {
        assert_eq!(ExprParser::new().parse("2 + 3 * 4"), Ok(14));
    }

    #[test]
    fn parentesis() {
        assert_eq!(ExprParser::new().parse("(2 + 3) * 4"), Ok(20));
    }

    #[test]
    fn asociatividad_izquierda() {
        assert_eq!(ExprParser::new().parse("10 - 3 - 2"), Ok(5));
    }

    #[test]
    fn entrada_invalida() {
        assert!(ExprParser::new().parse("2 + * 4").is_err());
    }
}
