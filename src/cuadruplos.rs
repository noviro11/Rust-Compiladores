// ============================================================================
//  GENERACIÓN DE CUÁDRUPLOS: pilas (operandos, operadores, tipos) y fila
// ============================================================================

use crate::cubo_semantico::{ErrorUsuario, Operador, Tipo, error_en, validar};
use crate::memoria::MemoriaVirtual;
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
    Goto,    // salto incondicional
    GotoF,   // salto si la condición es falsa
    Era,     // reservar la memoria de una función antes de llamarla
    Param,   // mandar un argumento a la función
    Gosub,   // brincar al código de la función (y recordar a dónde regresar)
    EndFunc, // fin de una función: regresar a quien la llamó
    End,     // fin del programa
}

/// Una llamada que se está traduciendo: hace falta para revisar sus argumentos.
/// Es una pila porque puede haber llamadas dentro de argumentos: f(g(x)).
#[derive(Debug)]
pub struct LlamadaPendiente {
    nombre: String,
    inicio: usize,          // cuádruplo donde empieza la función
    parametros: Vec<Tipo>,  // tipos que espera
    argumentos: usize,      // cuántos argumentos llevamos
}

/// Un cuádruplo: (operador, operando izquierdo, operando derecho, resultado).
/// Cada casilla es una dirección virtual. None = casilla vacía (se imprime como "_").
#[derive(Debug, Clone)]
pub struct Cuadruplo {
    pub op: OpCuadruplo,
    pub izq: Option<usize>,
    pub der: Option<usize>,
    pub res: Option<usize>,
}

/// Las 3 pilas, la fila y la memoria virtual (que reparte direcciones).
#[derive(Debug)]
pub struct GeneradorCuadruplos {
    pub pila_operandos: Vec<usize>,     // PilaO (ahora guarda direcciones)
    pub pila_operadores: Vec<Operador>, // POper
    pub pila_tipos: Vec<Tipo>,          // PTipos (en paralelo con PilaO)
    pub fila: Vec<Cuadruplo>,           // Fila de cuádruplos
    pub pila_saltos: Vec<usize>,        // PSaltos: números de cuádruplo pendientes de rellenar o de regreso
    pila_llamadas: Vec<LlamadaPendiente>, // llamadas a función que se están traduciendo
    pub memoria: MemoriaVirtual,        // da direcciones a temporales, constantes y variables
}

type ErrorParser = ParseError<usize, Token, ErrorUsuario>;

impl GeneradorCuadruplos {
    pub fn nuevo() -> Self {
        GeneradorCuadruplos {
            pila_operandos: Vec::new(),
            pila_operadores: Vec::new(),
            pila_tipos: Vec::new(),
            fila: Vec::new(),
            pila_saltos: Vec::new(),
            pila_llamadas: Vec::new(),
            memoria: MemoriaVirtual::nueva(),
        }
    }

    /// Mete un operando (variable, constante o temporal) a PilaO y su tipo a PTipos.
    pub fn meter_operando(&mut self, operando: usize, tipo: Tipo) {
        self.pila_operandos.push(operando);
        self.pila_tipos.push(tipo);
    }

    /// Mete un operador a POper.
    pub fn meter_operador(&mut self, op: Operador) {
        self.pila_operadores.push(op);
    }

    /// Saca el operando del tope de PilaO junto con su tipo de PTipos.
    fn sacar_operando(&mut self) -> (usize, Tipo) {
        let operando = self.pila_operandos.pop().expect("PilaO vacía");
        let tipo = self.pila_tipos.pop().expect("PTipos vacía");
        (operando, tipo)
    }

    /// Pide a la memoria la dirección de un temporal nuevo del tipo dado (5000+, 6000+ o 7000+).
    fn nuevo_temporal(&mut self, tipo: Tipo, pos: usize) -> Result<usize, ErrorParser> {
        self.memoria.nuevo_temporal(tipo).map_err(|m| error_en(pos, m))
    }

    /// Agrega un cuádruplo al final de la fila.
    fn generar(
        &mut self,
        op: OpCuadruplo,
        izq: Option<usize>,
        der: Option<usize>,
        res: Option<usize>,
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
        let temporal = self.nuevo_temporal(tipo, pos)?; // el temporal es del tipo que dio el cubo
        self.generar(
            OpCuadruplo::Operacion(op),
            Some(izq),
            Some(der),
            Some(temporal),
        );
        self.meter_operando(temporal, tipo);
        Ok(tipo)
    }

    /// Signo negativo: -x se traduce como (-, 0, x, temporal).
    pub fn generar_negativo(&mut self, pos: usize) -> Result<Tipo, ErrorParser> {
        let (valor, tipo_valor) = self.sacar_operando();
        let tipo = validar(Operador::Resta, Tipo::Int, tipo_valor, pos)?;
        let cero = self.memoria.constante_int(0).map_err(|m| error_en(pos, m))?; // el 0 también es constante
        let temporal = self.nuevo_temporal(tipo, pos)?;
        self.generar(
            OpCuadruplo::Operacion(Operador::Resta),
            Some(cero),
            Some(valor),
            Some(temporal),
        );
        self.meter_operando(temporal, tipo);
        Ok(tipo)
    }

    /// Asignación: saca el valor de PilaO, valida con el cubo y genera (=, valor, _, destino).
    pub fn generar_asignacion(
        &mut self,
        destino: usize,
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

    /// print de un letrero: el texto se guarda como constante string y se genera (print, _, _, dirección).
    pub fn generar_print_texto(&mut self, texto: String, pos: usize) -> Result<(), ErrorParser> {
        let direccion = self.memoria.constante_string(&texto).map_err(|m| error_en(pos, m))?;
        self.generar(OpCuadruplo::Print, None, None, Some(direccion));
        Ok(())
    }

    /// return: saca el valor de PilaO, valida que se pueda regresar con el tipo
    /// de la función (mismo renglón del cubo que la asignación) y genera
    /// (return, valor, _, variable global de la función): copia el valor y sale de la función.
    pub fn generar_return(
        &mut self,
        tipo_funcion: Tipo,
        direccion_retorno: usize,
        pos: usize,
    ) -> Result<(), ErrorParser> {
        let (valor, tipo_valor) = self.sacar_operando();
        validar(Operador::Asignar, tipo_funcion, tipo_valor, pos)?;
        self.generar(OpCuadruplo::Return, Some(valor), None, Some(direccion_retorno));
        Ok(())
    }

    /// read: genera (read, _, _, variable).
    pub fn generar_read(&mut self, destino: usize) {
        self.generar(OpCuadruplo::Read, None, None, Some(destino));
    }

    // ----------------------- Estatutos no lineales -----------------------
    // Los cuádruplos se numeran desde 1 (igual que al imprimir la fila).

    /// Número que tendrá el próximo cuádruplo que se genere.
    pub fn siguiente(&self) -> usize {
        self.fila.len() + 1
    }

    /// Escribe el destino de un salto que se generó con la casilla vacía.
    fn rellenar(&mut self, numero: usize, destino: usize) {
        self.fila[numero - 1].res = Some(destino);
    }

    /// Después de la condición de un if o while: la condición debe ser bool,
    /// se genera (GOTOF, condición, _, ?) y se guarda su número en PSaltos para rellenarlo después.
    pub fn generar_gotof(&mut self, pos: usize) -> Result<(), ErrorParser> {
        let (condicion, tipo) = self.sacar_operando();
        if tipo != Tipo::Bool {
            return Err(error_en(
                pos,
                format!("la condición debe ser bool, pero es {:?}", tipo),
            ));
        }
        self.generar(OpCuadruplo::GotoF, Some(condicion), None, None);
        self.pila_saltos.push(self.fila.len()); // número del GOTOF que acabamos de generar
        Ok(())
    }

    /// Al llegar al else: se genera (GOTO, _, _, ?) para saltarse el else cuando el if fue verdadero,
    /// y el GOTOF pendiente se rellena para que caiga al inicio del else.
    pub fn generar_else(&mut self) {
        self.generar(OpCuadruplo::Goto, None, None, None);
        let falso = self.pila_saltos.pop().expect("PSaltos vacía");
        self.pila_saltos.push(self.fila.len()); // ahora queda pendiente el GOTO
        self.rellenar(falso, self.siguiente());
    }

    /// Fin del if: el salto pendiente (GOTOF si no hubo else, GOTO si sí) apunta al siguiente cuádruplo.
    pub fn generar_fin_if(&mut self) {
        let fin = self.pila_saltos.pop().expect("PSaltos vacía");
        self.rellenar(fin, self.siguiente());
    }

    /// Al leer while: se guarda en PSaltos a dónde hay que regresar (donde empieza la condición).
    pub fn generar_inicio_while(&mut self) {
        self.pila_saltos.push(self.siguiente());
    }

    /// Fin del while: GOTO de regreso a la condición, y el GOTOF se rellena para salir del ciclo.
    pub fn generar_fin_while(&mut self) {
        let falso = self.pila_saltos.pop().expect("PSaltos vacía");
        let regreso = self.pila_saltos.pop().expect("PSaltos vacía");
        self.generar(OpCuadruplo::Goto, None, None, Some(regreso));
        self.rellenar(falso, self.siguiente());
    }
}

impl GeneradorCuadruplos {
    // ----------------------- Programa y funciones -----------------------

    /// Al leer "program id ;": el primer cuádruplo es (GOTO, _, _, ?) hacia main,
    /// porque el código de las funciones va primero y no se debe ejecutar al arrancar.
    pub fn generar_goto_main(&mut self) {
        self.generar(OpCuadruplo::Goto, None, None, None);
        self.pila_saltos.push(self.fila.len());
    }

    /// Al leer "main": se rellena el GOTO del inicio para que caiga aquí.
    pub fn rellenar_goto_main(&mut self) {
        let goto_main = self.pila_saltos.pop().expect("PSaltos vacía");
        self.rellenar(goto_main, self.siguiente());
    }

    /// Fin de una función: (ENDFUNC, _, _, _).
    pub fn generar_endfunc(&mut self) {
        self.generar(OpCuadruplo::EndFunc, None, None, None);
    }

    /// Fin del programa: (END, _, _, _).
    pub fn generar_end(&mut self) {
        self.generar(OpCuadruplo::End, None, None, None);
    }

    /// Al leer "id (" de una llamada: (ERA, _, _, inicio de la función) y se abre la llamada.
    pub fn generar_era(&mut self, nombre: String, inicio: usize, parametros: Vec<Tipo>) {
        self.generar(OpCuadruplo::Era, None, None, Some(inicio));
        self.pila_llamadas.push(LlamadaPendiente {
            nombre,
            inicio,
            parametros,
            argumentos: 0,
        });
    }

    /// Después de cada argumento: saca su valor de PilaO, revisa que su tipo se pueda
    /// asignar al parámetro que le toca y genera (PARAM, valor, _, número de parámetro).
    pub fn generar_param(&mut self, pos: usize) -> Result<(), ErrorParser> {
        let (valor, tipo_valor) = self.sacar_operando();
        let llamada = self.pila_llamadas.last_mut().expect("no hay llamada abierta");
        llamada.argumentos += 1;
        let numero = llamada.argumentos; // 1 = primer parámetro, 2 = segundo, ...
        let Some(&tipo_parametro) = llamada.parametros.get(numero - 1) else {
            return Err(error_en(
                pos,
                format!(
                    "'{}' recibe {} parámetro(s) y se le mandaron más",
                    llamada.nombre,
                    llamada.parametros.len()
                ),
            ));
        };
        validar(Operador::Asignar, tipo_parametro, tipo_valor, pos)?;
        self.generar(OpCuadruplo::Param, Some(valor), None, Some(numero));
        Ok(())
    }

    /// Al cerrar ")" de la llamada: revisa que no falten argumentos y genera (GOSUB, _, _, inicio).
    pub fn generar_gosub(&mut self, pos: usize) -> Result<(), ErrorParser> {
        let llamada = self.pila_llamadas.pop().expect("no hay llamada abierta");
        if llamada.argumentos < llamada.parametros.len() {
            return Err(error_en(
                pos,
                format!(
                    "'{}' recibe {} parámetro(s) y se le mandaron {}",
                    llamada.nombre,
                    llamada.parametros.len(),
                    llamada.argumentos
                ),
            ));
        }
        self.generar(OpCuadruplo::Gosub, None, None, Some(llamada.inicio));
        Ok(())
    }

    /// Llamada dentro de una expresión: copia el valor de la variable global de la función
    /// a un temporal (=, global, _, temporal) y mete el temporal a PilaO.
    pub fn generar_copia_retorno(
        &mut self,
        direccion_retorno: usize,
        tipo: Tipo,
        pos: usize,
    ) -> Result<Tipo, ErrorParser> {
        let temporal = self.nuevo_temporal(tipo, pos)?;
        self.generar(
            OpCuadruplo::Operacion(Operador::Asignar),
            Some(direccion_retorno),
            None,
            Some(temporal),
        );
        self.meter_operando(temporal, tipo);
        Ok(tipo)
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
            OpCuadruplo::Goto => "GOTO",
            OpCuadruplo::GotoF => "GOTOF",
            OpCuadruplo::Era => "ERA",
            OpCuadruplo::Param => "PARAM",
            OpCuadruplo::Gosub => "GOSUB",
            OpCuadruplo::EndFunc => "ENDFUNC",
            OpCuadruplo::End => "END",
        };
        write!(f, "{}", simbolo)
    }
}

impl fmt::Display for Cuadruplo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let casilla = |c: &Option<usize>| c.map(|d| d.to_string()).unwrap_or("_".to_string());
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
