// ============================================================================
//  ANALIZADOR: une el scanner (lexer.rs) con el parser (baby_duck.lalrpop)
//  y traduce los errores de LALRPOP a mensajes con línea y columna.
// ============================================================================

use crate::baby_duck::ProgramaParser;
use crate::cubo_semantico::ErrorUsuario;
use crate::directorio::DirFunciones;
use crate::lexer::{Lexer, Token};
use lalrpop_util::ParseError;
use std::fmt;

/// Los tres tipos de error: léxico, de sintaxis y semántico.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorBD {
    Lexico {
        linea: usize,
        columna: usize,
        mensaje: String,
    },
    Sintactico {
        linea: usize,
        columna: usize,
        mensaje: String,
    },
    Semantico {
        linea: usize,
        columna: usize,
        mensaje: String,
    },
}

impl fmt::Display for ErrorBD {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorBD::Lexico {
                linea,
                columna,
                mensaje,
            } => {
                write!(
                    f,
                    "Error léxico (línea {}, columna {}): {}",
                    linea, columna, mensaje
                )
            }
            ErrorBD::Sintactico {
                linea,
                columna,
                mensaje,
            } => {
                write!(
                    f,
                    "Error de sintaxis (línea {}, columna {}): {}",
                    linea, columna, mensaje
                )
            }
            ErrorBD::Semantico {
                linea,
                columna,
                mensaje,
            } => {
                write!(
                    f,
                    "Error semántico (línea {}, columna {}): {}",
                    linea, columna, mensaje
                )
            }
        }
    }
}

/// Analiza un programa completo. Si todo está bien, regresa el Directorio lleno.
pub fn analizar(fuente: &str) -> Result<DirFunciones, ErrorBD> {
    let mut dir = DirFunciones::nuevo(""); // vacío: el nombre real lo pone EncabezadoPrograma
    let lexer = Lexer::new(fuente).map(|r| r.map_err(ErrorUsuario::Lexico)); // 1) el scanner produce tokens…
    let parser = ProgramaParser::new(); // 2) …y el parser los consume
    parser
        .parse(&mut dir, lexer)
        .map_err(|e| traducir_error(fuente, e))?;
    Ok(dir)
}

/// Convierte una posición (índice en el texto) en (línea, columna), ambas desde 1.
fn linea_columna(fuente: &str, pos: usize) -> (usize, usize) {
    let antes = &fuente[..pos.min(fuente.len())];
    let linea = antes.matches('\n').count() + 1;
    let columna = antes.chars().rev().take_while(|&c| c != '\n').count() + 1;
    (linea, columna)
}

/// Lista de tokens esperados, ya sin comillas extra: "id", "(", ...
fn esperados(lista: &[String]) -> String {
    if lista.is_empty() {
        return "fin del archivo".to_string(); // ya no debería venir nada más
    }
    lista
        .iter()
        .map(|s| s.trim_matches('"'))
        .collect::<Vec<_>>()
        .join(", ")
}

/// LALRPOP regresa un ParseError con varios casos posibles; cada uno se
/// convierte en un mensaje entendible.
fn traducir_error(fuente: &str, e: ParseError<usize, Token, ErrorUsuario>) -> ErrorBD {
    match e {
        // El scanner encontró texto que no es ningún token.
        ParseError::User {
            error: ErrorUsuario::Lexico(error),
        } => {
            let (linea, columna) = linea_columna(fuente, error.inicio);
            ErrorBD::Lexico {
                linea,
                columna,
                mensaje: format!("token inválido '{}'", error.texto),
            }
        }
        // Un punto neurálgico del .lalrpop detectó un error semántico.
        ParseError::User {
            error: ErrorUsuario::Semantico(pos, mensaje),
        } => {
            let (linea, columna) = linea_columna(fuente, pos);
            ErrorBD::Semantico {
                linea,
                columna,
                mensaje,
            }
        }
        // Llegó un token que la gramática no permite en ese lugar.
        ParseError::UnrecognizedToken {
            token: (inicio, tok, _),
            expected,
        } => {
            let (linea, columna) = linea_columna(fuente, inicio);
            ErrorBD::Sintactico {
                linea,
                columna,
                mensaje: format!(
                    "no se esperaba {:?}; se esperaba: {}",
                    tok,
                    esperados(&expected)
                ),
            }
        }
        // El archivo se acabó pero a la gramática le faltaban tokens.
        ParseError::UnrecognizedEof { location, expected } => {
            let (linea, columna) = linea_columna(fuente, location);
            ErrorBD::Sintactico {
                linea,
                columna,
                mensaje: format!(
                    "el programa terminó antes de tiempo; se esperaba: {}",
                    esperados(&expected)
                ),
            }
        }
        // Sobró algo después de un programa completo (ej. texto después de `end`).
        ParseError::ExtraToken {
            token: (inicio, tok, _),
        } => {
            let (linea, columna) = linea_columna(fuente, inicio);
            ErrorBD::Sintactico {
                linea,
                columna,
                mensaje: format!("token sobrante {:?}", tok),
            }
        }
        // Solo lo produce el lexer integrado de LALRPOP; con Logos no ocurre.
        ParseError::InvalidToken { location } => {
            let (linea, columna) = linea_columna(fuente, location);
            ErrorBD::Lexico {
                linea,
                columna,
                mensaje: "token inválido".to_string(),
            }
        }
    }
}
