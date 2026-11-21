//  DIRECTORIO DE FUNCIONES y TABLAS DE VARIABLES

use crate::cubo_semantico::{Tipo, TipoRetorno};
use crate::memoria::{MemoriaVirtual, Recursos};
use std::collections::HashMap;

// Lo que se guarda de cada variable en su Tabla de Variables.
#[derive(Debug, Clone, Copy)]
pub struct Variable {
    pub tipo: Tipo,
    pub direccion: usize, // su dirección virtual
}

// Las partes de la funcion (incluye la "global" osea del programa completo)
#[derive(Debug)]

pub struct Funcion {
    pub tipo_retorno: TipoRetorno,        // Void o Valor(Int/Float)
    pub parametros: Vec<Tipo>,            // parametros de entrada
    pub variables: HashMap<String, Variable>, // su Tabla de Variables: nombre → (tipo, dirección)
    pub direccion_retorno: Option<usize>, // variable global donde deja su valor (None si es void)
    pub inicio: usize,                    // número del cuádruplo donde empieza su código
    pub recursos: Recursos,               // cuántas locales y temporales usa (tamaño para el ERA)
}
// Constructor
impl Funcion {
    // nueva función: sin parámetros ni variables todavía.
    fn nueva(tipo_retorno: TipoRetorno) -> Self {
        Funcion {
            tipo_retorno,
            parametros: Vec::new(),
            variables: HashMap::new(),
            direccion_retorno: None,
            inicio: 0,                     // se pone al leer el encabezado de la función
            recursos: Recursos::default(), // se llena al terminar la función
        }
    }
}

// El directorio completo.
#[derive(Debug)]
pub struct DirFunciones {
    pub funciones: HashMap<String, Funcion>, // nombre → función
    pub global: String,                      // nombre del programa (alcance global)
    pub actual: String,                      // función en la que estamos ahora
}
// Constructor
impl DirFunciones {
    // Crea el directorio de todas las funciones, se inicializa con solo una (la del programa global)
    pub fn nuevo(nombre_programa: &str) -> Self {
        let mut funciones = HashMap::new();
        funciones.insert(
            nombre_programa.to_string(),
            Funcion::nueva(TipoRetorno::Void),
        );
        DirFunciones {
            funciones,
            global: nombre_programa.to_string(),
            actual: nombre_programa.to_string(),
        }
    }
    // Da de alta una función y cambia el alcance actual a esa
    pub fn agregar_funcion(
        &mut self,
        nombre: &str,
        tipo: TipoRetorno,
        memoria: &mut MemoriaVirtual,
    ) -> Result<(), String> {
        if self.funciones.contains_key(nombre) {
            return Err(format!("la función '{}' ya fue declarada", nombre));
        }
        let mut funcion = Funcion::nueva(tipo);
        // Si regresa un valor, se le reserva una dirección GLOBAL donde lo dejará el return.
        if let TipoRetorno::Valor(t) = tipo {
            funcion.direccion_retorno = Some(memoria.nueva_variable(t, true)?);
        }
        self.funciones.insert(nombre.to_string(), funcion);
        self.actual = nombre.to_string();
        Ok(())
    }

    // Guarda una variable en la tabla del alcance ACTUAL.
    pub fn agregar_variable(
        &mut self,
        nombre: &str,
        tipo: Tipo,
        memoria: &mut MemoriaVirtual,
    ) -> Result<(), String> {
        let es_global = self.actual == self.global;
        let tabla = &mut self
            .funciones
            .get_mut(&self.actual)
            .expect("el alcance actual siempre existe")
            .variables;
        // Solo revisa esa tabla ya que una variable local puede llamarse igual que una global.
        if tabla.contains_key(nombre) {
            return Err(format!(
                "la variable '{}' ya fue declarada en '{}'",
                nombre, self.actual
            ));
        }
        let direccion = memoria.nueva_variable(tipo, es_global)?; // global → 1000/2000, local → 3000/4000
        tabla.insert(nombre.to_string(), Variable { tipo, direccion });
        Ok(())
    }

    // Un parámetro es un parametro que ademas se anota como variable local
    pub fn agregar_parametro(
        &mut self,
        nombre: &str,
        tipo: Tipo,
        memoria: &mut MemoriaVirtual,
    ) -> Result<(), String> {
        self.agregar_variable(nombre, tipo, memoria)?;
        self.funciones
            .get_mut(&self.actual)
            .expect("el alcance actual siempre existe")
            .parametros
            .push(tipo);
        Ok(())
    }

    // Busca primero en la tabla actual y luego en la global (la local tiene prioridad sobre la global).
    pub fn buscar_variable(&self, nombre: &str) -> Result<Variable, String> {
        if let Some(variable) = self.funciones[&self.actual].variables.get(nombre) {
            return Ok(*variable);
        }
        if let Some(variable) = self.funciones[&self.global].variables.get(nombre) {
            return Ok(*variable);
        }
        Err(format!("la variable '{}' no fue declarada", nombre))
    }

    // Busca una función por nombre.
    pub fn buscar_funcion(&self, nombre: &str) -> Result<&Funcion, String> {
        self.funciones
            .get(nombre)
            .ok_or(format!("la función '{}' no fue declarada", nombre))
    }

    /// Tipo que debe regresar la función actual. Error si estamos en main
    /// (alcance global) o si la función es void.
    pub fn retorno_actual(&self) -> Result<(Tipo, usize), String> {
        if self.actual == self.global {
            return Err("return solo se puede usar dentro de una función".to_string());
        }
        let funcion = &self.funciones[&self.actual];
        match (funcion.tipo_retorno, funcion.direccion_retorno) {
            (TipoRetorno::Valor(tipo), Some(direccion)) => Ok((tipo, direccion)),
            _ => Err(format!(
                "la función '{}' es void y no puede regresar un valor",
                self.actual
            )),
        }
    }

    // La función en la que estamos (o la global, en main), para modificar su inicio o recursos.
    pub fn funcion_actual_mut(&mut self) -> &mut Funcion {
        self.funciones
            .get_mut(&self.actual)
            .expect("el alcance actual siempre existe")
    }

    // Al terminar una función, regresa al alcance global.
    pub fn salir_funcion(&mut self) {
        self.actual = self.global.clone();
    }
}
