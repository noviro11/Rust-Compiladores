// ============================================================================
//  GENERACIÓN DE CUÁDRUPLOS: pilas (operandos, operadores, tipos) y fila
// ============================================================================

use crate::cubo_semantico::{ErrorUsuario, Operador, Tipo, validar};
use crate::lexer::Token;
use lalrpop_util::ParseError;
use std::fmt;

/// La operación de un cuádruplo: una del cubo (+, -, *, /, >, <, !=, =), print o read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCuadruplo {
    Operacion(Operador),
    Print,
    Read,
    Return,
}

/// Un cuádruplo: (operador, operando izquierdo, operando derecho, resultado).
/// None = casilla vacía (se imprime como "_").
#[derive(Debug, Clone)]
pub struct Cuadruplo {
    pub op: OpCuadruplo,
    pub izq: Option<String>,
    pub der: Option<String>,
    pub res: Option<String>,
}

/// Las 3 pilas, la fila y el contador de temporales.
#[derive(Debug)]
pub struct GeneradorCuadruplos {
    pub pila_operandos: Vec<String>,    // PilaO
    pub pila_operadores: Vec<Operador>, // POper
    pub pila_tipos: Vec<Tipo>,          // PTipos (en paralelo con PilaO)
    pub fila: Vec<Cuadruplo>,           // Fila de cuádruplos
    temporales: usize,                  // cuántos temporales se han creado (t1, t2, ...)
}

type ErrorParser = ParseError<usize, Token, ErrorUsuario>;

impl GeneradorCuadruplos {
    pub fn nuevo() -> Self {
        GeneradorCuadruplos {
            pila_operandos: Vec::new(),
            pila_operadores: Vec::new(),
            pila_tipos: Vec::new(),
            fila: Vec::new(),
            temporales: 0,
        }
    }

    /// Mete un operando (variable, constante o temporal) a PilaO y su tipo a PTipos.
    pub fn meter_operando(&mut self, operando: String, tipo: Tipo) {
        self.pila_operandos.push(operando);
        self.pila_tipos.push(tipo);
    }

    /// Mete un operador a POper.
    pub fn meter_operador(&mut self, op: Operador) {
        self.pila_operadores.push(op);
    }

    /// Saca el operando del tope de PilaO junto con su tipo de PTipos.
    fn sacar_operando(&mut self) -> (String, Tipo) {
        let operando = self.pila_operandos.pop().expect("PilaO vacía");
        let tipo = self.pila_tipos.pop().expect("PTipos vacía");
        (operando, tipo)
    }

    /// Crea el nombre de un temporal nuevo: t1, t2, t3, ...
    fn nuevo_temporal(&mut self) -> String {
        self.temporales += 1;
        format!("t{}", self.temporales)
    }

    /// Agrega un cuádruplo al final de la fila.
    fn generar(
        &mut self,
        op: OpCuadruplo,
        izq: Option<String>,
        der: Option<String>,
        res: Option<String>,
    ) {
        self.fila.push(Cuadruplo { op, izq, der, res });
    }

    /// Algoritmo de expresiones: saca 2 operandos y 1 operador, valida con el cubo,
    /// genera (op, izq, der, temporal) y mete el temporal a PilaO.
    pub fn generar_operacion(&mut self, pos: usize) -> Result<Tipo, ErrorParser> {
        let (der, tipo_der) = self.sacar_operando();
        let (izq, tipo_izq) = self.sacar_operando();
        let op = self.pila_operadores.pop().expect("POper vacía");
        let tipo = validar(op, tipo_izq, tipo_der, pos)?;
        let temporal = self.nuevo_temporal();
        self.generar(
            OpCuadruplo::Operacion(op),
            Some(izq),
            Some(der),
            Some(temporal.clone()),
        );
        self.meter_operando(temporal, tipo);
        Ok(tipo)
    }

    /// Signo negativo: -x se traduce como (-, 0, x, temporal).
    pub fn generar_negativo(&mut self, pos: usize) -> Result<Tipo, ErrorParser> {
        let (valor, tipo_valor) = self.sacar_operando();
        let tipo = validar(Operador::Resta, Tipo::Int, tipo_valor, pos)?;
        let temporal = self.nuevo_temporal();
        self.generar(
            OpCuadruplo::Operacion(Operador::Resta),
            Some("0".to_string()),
            Some(valor),
            Some(temporal.clone()),
        );
        self.meter_operando(temporal, tipo);
        Ok(tipo)
    }

    /// Asignación: saca el valor de PilaO, valida con el cubo y genera (=, valor, _, destino).
    pub fn generar_asignacion(
        &mut self,
        destino: String,
        tipo_destino: Tipo,
        pos: usize,
    ) -> Result<(), ErrorParser> {
        let (valor, tipo_valor) = self.sacar_operando();
        validar(Operador::Asignar, tipo_destino, tipo_valor, pos)?;
        self.generar(
            OpCuadruplo::Operacion(Operador::Asignar),
            Some(valor),
            None,
            Some(destino),
        );
        Ok(())
    }

    /// print de una expresión: saca su resultado de PilaO y genera (print, _, _, valor).
    pub fn generar_print_expresion(&mut self) {
        let (valor, _) = self.sacar_operando();
        self.generar(OpCuadruplo::Print, None, None, Some(valor));
    }

    /// print de un letrero: genera (print, _, _, "texto").
    pub fn generar_print_texto(&mut self, texto: String) {
        self.generar(
            OpCuadruplo::Print,
            None,
            None,
            Some(format!("\"{}\"", texto)),
        );
    }

    /// return: saca el valor de PilaO, valida que se pueda regresar con el tipo
    /// de la función (mismo renglón del cubo que la asignación) y genera (return, _, _, valor).
    pub fn generar_return(&mut self, tipo_funcion: Tipo, pos: usize) -> Result<(), ErrorParser> {
        let (valor, tipo_valor) = self.sacar_operando();
        validar(Operador::Asignar, tipo_funcion, tipo_valor, pos)?;
        self.generar(OpCuadruplo::Return, None, None, Some(valor));
        Ok(())
    }

    /// read: genera (read, _, _, variable).
    pub fn generar_read(&mut self, destino: String) {
        self.generar(OpCuadruplo::Read, None, None, Some(destino));
    }
}

// ----------------------- Cómo se imprime -----------------------

impl fmt::Display for OpCuadruplo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let simbolo = match self {
            OpCuadruplo::Operacion(Operador::Suma) => "+",
            OpCuadruplo::Operacion(Operador::Resta) => "-",
            OpCuadruplo::Operacion(Operador::Mult) => "*",
            OpCuadruplo::Operacion(Operador::Div) => "/",
            OpCuadruplo::Operacion(Operador::Mayor) => ">",
            OpCuadruplo::Operacion(Operador::Menor) => "<",
            OpCuadruplo::Operacion(Operador::Diferente) => "!=",
            OpCuadruplo::Operacion(Operador::Asignar) => "=",
            OpCuadruplo::Print => "print",
            OpCuadruplo::Read => "read",
            OpCuadruplo::Return => "return",
        };
        write!(f, "{}", simbolo)
    }
}

impl fmt::Display for Cuadruplo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let casilla = |c: &Option<String>| c.clone().unwrap_or("_".to_string());
        write!(
            f,
            "({}, {}, {}, {})",
            self.op,
            casilla(&self.izq),
            casilla(&self.der),
            casilla(&self.res)
        )
    }
}
