mod analizador;
mod cuadruplos;
mod cubo_semantico;
mod directorio;
mod lexer;

use lalrpop_util::lalrpop_mod;
use std::{env, fs, process};

// Incluye el parser que LALRPOP generó a partir de src/baby_duck.lalrpop
lalrpop_mod!(pub baby_duck);

fn main() {
    // Uso: cargo run -- <archivo.bd> [--tokens]
    let args: Vec<String> = env::args().collect();
    let ruta = args.get(1).unwrap_or_else(|| {
        eprintln!("uso: cargo run -- <archivo.bd> [--tokens]");
        process::exit(2);
    });
    let fuente = fs::read_to_string(ruta).expect("no se pudo leer el archivo");

    // Opcional: imprimir los tokens (lo que hacía la Parte 1)
    for resultado in lexer::Lexer::new(&fuente) {
        match resultado {
            Ok((inicio, token, fin)) => println!("{:>4}..{:<4} {:?}", inicio, fin, token),
            Err(e) => {
                println!("ERROR LÉXICO: {}", e);
                break;
            }
        }
    }
    println!("--------------------------------");

    // Parte 2: scanner + parser
    match analizador::analizar(&fuente) {
        Ok((dir, generador)) => {
            println!(
                "OK: '{}' es léxica, sintáctica y semánticamente correcto.",
                ruta
            );
            println!("{:#?}", dir); // imprime el Directorio de Funciones completo
            println!("--------- Fila de cuádruplos ---------");
            for (i, cuadruplo) in generador.fila.iter().enumerate() {
                println!("{:>3}. {}", i + 1, cuadruplo);
            }
        }
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    }
}
