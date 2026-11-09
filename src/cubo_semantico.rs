#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tipo {
    Int,
    Float,
    Bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TipoRetorno {
    Void,
    Valor(Tipo),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operador {
    Suma,
    Resta,
    Mult,
    Div,
    Mayor,
    Menor,
    Diferente,
    Asignar,
}

pub fn resultado_operacion(op: Operador, izq: Tipo, der: Tipo) -> Option<Tipo> {
    use Operador::*;
    use Tipo::*;
    match (op, izq, der) {
        // +  -  *
        (Suma | Resta | Mult, Int, Int) => Some(Int),
        (Suma | Resta | Mult, Int, Float) => Some(Float),
        (Suma | Resta | Mult, Float, Int) => Some(Float),
        (Suma | Resta | Mult, Float, Float) => Some(Float),

        // /
        (Div, Int | Float, Int | Float) => Some(Float),

        // >  <  !=
        (Mayor | Menor | Diferente, Int | Float, Int | Float) => Some(Bool),

        // =   (izq = variable destino, der = valor que se asigna)
        (Asignar, Int, Int) => Some(Int),
        (Asignar, Float, Int) => Some(Float),
        (Asignar, Float, Float) => Some(Float),

        // todo lo demás es error: int ← float, cualquier cosa con bool, etc.
        _ => None,
    }
}

// Errores
use crate::lexer::{LexicalError, Token};
use lalrpop_util::ParseError;

// Los errores que mandamos nosotros al parser: del scanner o semánticos.
#[derive(Debug)]
pub enum ErrorUsuario {
    Lexico(LexicalError),
    Semantico(usize, String), // (posición, mensaje)
}

// Igual que resultado_operacion, pero si es None regresa un error semántico.
pub fn validar(
    op: Operador,
    izq: Tipo,
    der: Tipo,
    pos: usize,
) -> Result<Tipo, ParseError<usize, Token, ErrorUsuario>> {
    resultado_operacion(op, izq, der).ok_or(ParseError::User {
        error: ErrorUsuario::Semantico(
            pos,
            format!("tipos incompatibles: {:?} {:?} {:?}", izq, op, der),
        ),
    })
}

// Convierte un mensaje (String) en el error semántico que entiende LALRPOP, con la posición donde ocurrió.
pub fn error_en(pos: usize, mensaje: String) -> ParseError<usize, Token, ErrorUsuario> {
    ParseError::User {
        error: ErrorUsuario::Semantico(pos, mensaje),
    }
}
