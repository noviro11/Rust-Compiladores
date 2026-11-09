//  DIRECTORIO DE FUNCIONES y TABLAS DE VARIABLES

use crate::cubo_semantico::{Tipo, TipoRetorno};
use std::collections::HashMap;

// Las partes de la funcion (incluye la "global" osea del programa completo)
#[derive(Debug)]

pub struct Funcion {
    pub tipo_retorno: TipoRetorno,        // Void o Valor(Int/Float)
    pub parametros: Vec<Tipo>,            // parametros de entrada
    pub variables: HashMap<String, Tipo>, // su Tabla de Variables: nombre → tipo
}
// Constructor
impl Funcion {
    // nueva función: sin parámetros ni variables todavía.
    fn nueva(tipo_retorno: TipoRetorno) -> Self {
        Funcion {
            tipo_retorno,
            parametros: Vec::new(),
            variables: HashMap::new(),
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
    pub fn agregar_funcion(&mut self, nombre: &str, tipo: TipoRetorno) -> Result<(), String> {
        if self.funciones.contains_key(nombre) {
            return Err(format!("la función '{}' ya fue declarada", nombre));
        }
        self.funciones
            .insert(nombre.to_string(), Funcion::nueva(tipo));
        self.actual = nombre.to_string();
        Ok(())
    }

    // Guarda una variable en la tabla del alcance ACTUAL.
    pub fn agregar_variable(&mut self, nombre: &str, tipo: Tipo) -> Result<(), String> {
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
        tabla.insert(nombre.to_string(), tipo);
        Ok(())
    }

    // Un parámetro es un parametro que ademas se anota como variable local
    pub fn agregar_parametro(&mut self, nombre: &str, tipo: Tipo) -> Result<(), String> {
        self.agregar_variable(nombre, tipo)?;
        self.funciones
            .get_mut(&self.actual)
            .expect("el alcance actual siempre existe")
            .parametros
            .push(tipo);
        Ok(())
    }

    // Busca primero en la tabla actual y luego en la global (la local tiene prioridad sobre la global).
    pub fn buscar_variable(&self, nombre: &str) -> Result<Tipo, String> {
        if let Some(tipo) = self.funciones[&self.actual].variables.get(nombre) {
            return Ok(*tipo);
        }
        if let Some(tipo) = self.funciones[&self.global].variables.get(nombre) {
            return Ok(*tipo);
        }
        Err(format!("la variable '{}' no fue declarada", nombre))
    }

    // Busca una función por nombre.
    pub fn buscar_funcion(&self, nombre: &str) -> Result<&Funcion, String> {
        self.funciones
            .get(nombre)
            .ok_or(format!("la función '{}' no fue declarada", nombre))
    }

    // Al terminar una función, regresa al alcance global.
    pub fn salir_funcion(&mut self) {
        self.actual = self.global.clone();
    }
}
