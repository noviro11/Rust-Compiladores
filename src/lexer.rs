use logos::{Logos, SpannedIter};
use std::fmt;

// ---------------------------------------------------------------------------
// 1) Los TOKENS. Cada variante del enum es un tipo de token.
//    #[token("x")]  = cadena literal exacta.
//    #[regex(r"..")] = expresión regular.
// ---------------------------------------------------------------------------
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")] // espacios, tabs y saltos de línea se ignoran
pub enum Token {
    // ----- Palabras reservadas -----
    #[token("program")]
    Program,
    #[token("main")]
    Main,
    #[token("end")]
    End,
    #[token("var")]
    Var,
    #[token("int")]
    Int,
    #[token("float")]
    Float,
    #[token("void")]
    Void,
    #[token("print")]
    Print,
    #[token("read")]
    Read,
    #[token("while")]
    While,
    #[token("do")]
    Do,
    #[token("if")]
    If,
    #[token("else")]
    Else,

    // ----- Operadores -----
    #[token("=")]
    Asignar,
    #[token("+")]
    Mas,
    #[token("-")]
    Menos,
    #[token("*")]
    Por,
    #[token("/")]
    Entre,
    #[token(">")]
    Mayor,
    #[token("<")]
    Menor,
    #[token("!=")]
    Diferente,

    // ----- Delimitadores -----
    #[token(";")]
    PuntoComa,
    #[token(",")]
    Coma,
    #[token(":")]
    DosPuntos,
    #[token("(")]
    ParIzq,
    #[token(")")]
    ParDer,
    #[token("{")]
    LlaveIzq,
    #[token("}")]
    LlaveDer,
    #[token("[")]
    CorIzq,
    #[token("]")]
    CorDer,

    // ----- Tokens que llevan un VALOR -----
    // El closure |lex| ... recibe el lexer; lex.slice() es el texto reconocido.
    // Lo que regresa el closure se guarda dentro de la variante.

    // id: una letra seguida de letras, dígitos o '_'
    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Id(String),

    // cte_int: uno o más dígitos. Si no cabe en i64, parse() falla → error léxico.
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    CteInt(i64),

    // cte_float: dígitos, punto, dígitos  (ej. 3.14 — no se aceptan "3." ni ".5")
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    CteFloat(f64),

    // cte.string: comillas dobles, cualquier cosa excepto comillas o salto de línea, comillas
    // Se guarda el texto SIN las comillas.
    #[regex(r#""[^"\n]*""#, |lex| { let s = lex.slice(); s[1..s.len() - 1].to_string() })]
    CteString(String),
}

// Para imprimir tokens de forma legible (lo usan los mensajes de error).
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ---------------------------------------------------------------------------
// 2) El ERROR léxico: un pedazo de texto que no coincide con ninguna regex.
//    Guardamos dónde está y qué texto era, para reportarlo bien.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct LexicalError {
    pub inicio: usize,
    pub fin: usize,
    pub texto: String,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "token inválido '{}' en la posición {}",
            self.texto, self.inicio
        )
    }
}

// ---------------------------------------------------------------------------
// 3) El ITERADOR que consume el parser.
//    Logos ya da (Result<Token, ()>, rango). Aquí solo lo convertimos al
//    formato que LALRPOP espera:  Result<(inicio, Token, fin), LexicalError>
// ---------------------------------------------------------------------------
pub type Spanned = Result<(usize, Token, usize), LexicalError>;

pub struct Lexer<'input> {
    fuente: &'input str,
    tokens: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(fuente: &'input str) -> Self {
        Lexer {
            fuente,
            tokens: Token::lexer(fuente).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned;

    fn next(&mut self) -> Option<Self::Item> {
        // tokens.next() regresa None cuando se acaba el texto.
        let (resultado, rango) = self.tokens.next()?;
        Some(match resultado {
            Ok(token) => Ok((rango.start, token, rango.end)),
            Err(()) => Err(LexicalError {
                inicio: rango.start,
                fin: rango.end,
                texto: self.fuente[rango.clone()].to_string(),
            }),
        })
    }
}
