// ============================================================================
//  MEMORIA VIRTUAL: reparte una dirección a cada variable, temporal y constante
// ============================================================================

use crate::cubo_semantico::Tipo;
use std::collections::HashMap;

// Cada segmento tiene 1000 casillas.
const TAMANO: usize = 1000;

// Dónde empieza cada segmento (la distribución de direcciones virtuales).
pub const GLOBAL_INT: usize = 1000;
pub const GLOBAL_FLOAT: usize = 2000;
pub const LOCAL_INT: usize = 3000;
pub const LOCAL_FLOAT: usize = 4000;
pub const TEMP_INT: usize = 5000;
pub const TEMP_FLOAT: usize = 6000;
pub const TEMP_BOOL: usize = 7000;
pub const CTE_INT: usize = 8000;
pub const CTE_FLOAT: usize = 9000;
pub const CTE_STRING: usize = 10000;

// Un segmento: dónde empieza, cuál es la siguiente dirección libre y su nombre (para el error).
#[derive(Debug)]
struct Segmento {
    inicio: usize,
    siguiente: usize,
    nombre: &'static str,
}

impl Segmento {
    fn nuevo(inicio: usize, nombre: &'static str) -> Self {
        Segmento { inicio, siguiente: inicio, nombre }
    }

    // Entrega la siguiente dirección libre; error si ya se usaron las 1000.
    fn pedir(&mut self) -> Result<usize, String> {
        if self.siguiente >= self.inicio + TAMANO {
            return Err(format!("se acabó la memoria para {}", self.nombre));
        }
        let direccion = self.siguiente;
        self.siguiente += 1;
        Ok(direccion)
    }

    // Vuelve a empezar desde el inicio (para cada función nueva).
    fn reiniciar(&mut self) {
        self.siguiente = self.inicio;
    }

    // Cuántas direcciones se han usado en este segmento.
    fn usadas(&self) -> usize {
        self.siguiente - self.inicio
    }
}

// Cuántas casillas de cada tipo necesita una función (su "tamaño" para el ERA).
#[derive(Debug, Clone, Copy, Default)]
pub struct Recursos {
    pub locales_int: usize,
    pub locales_float: usize,
    pub temporales_int: usize,
    pub temporales_float: usize,
    pub temporales_bool: usize,
}

#[derive(Debug)]
pub struct MemoriaVirtual {
    global_int: Segmento,
    global_float: Segmento,
    local_int: Segmento,
    local_float: Segmento,
    temp_int: Segmento,
    temp_float: Segmento,
    temp_bool: Segmento,
    cte_int: Segmento,
    cte_float: Segmento,
    cte_string: Segmento,
    pub constantes: HashMap<String, usize>, // Tabla de constantes: valor (como texto) → dirección
}

impl MemoriaVirtual {
    pub fn nueva() -> Self {
        MemoriaVirtual {
            global_int: Segmento::nuevo(GLOBAL_INT, "globales int"),
            global_float: Segmento::nuevo(GLOBAL_FLOAT, "globales float"),
            local_int: Segmento::nuevo(LOCAL_INT, "locales int"),
            local_float: Segmento::nuevo(LOCAL_FLOAT, "locales float"),
            temp_int: Segmento::nuevo(TEMP_INT, "temporales int"),
            temp_float: Segmento::nuevo(TEMP_FLOAT, "temporales float"),
            temp_bool: Segmento::nuevo(TEMP_BOOL, "temporales bool"),
            cte_int: Segmento::nuevo(CTE_INT, "constantes int"),
            cte_float: Segmento::nuevo(CTE_FLOAT, "constantes float"),
            cte_string: Segmento::nuevo(CTE_STRING, "constantes string"),
            constantes: HashMap::new(),
        }
    }

    // Dirección para una variable: global (del programa) o local (de una función).
    pub fn nueva_variable(&mut self, tipo: Tipo, es_global: bool) -> Result<usize, String> {
        match (tipo, es_global) {
            (Tipo::Int, true) => self.global_int.pedir(),
            (Tipo::Float, true) => self.global_float.pedir(),
            (Tipo::Int, false) => self.local_int.pedir(),
            (Tipo::Float, false) => self.local_float.pedir(),
            (Tipo::Bool, _) => Err("no existen variables bool en BabyDuck".to_string()),
        }
    }

    // Dirección para un temporal (resultado de una operación).
    pub fn nuevo_temporal(&mut self, tipo: Tipo) -> Result<usize, String> {
        match tipo {
            Tipo::Int => self.temp_int.pedir(),
            Tipo::Float => self.temp_float.pedir(),
            Tipo::Bool => self.temp_bool.pedir(),
        }
    }

    // Constantes: si ya existe en la tabla se reutiliza su dirección, si no se le da una nueva.
    pub fn constante_int(&mut self, valor: i64) -> Result<usize, String> {
        let clave = valor.to_string(); // 5 → "5"
        Self::buscar_o_crear(&mut self.constantes, &mut self.cte_int, clave)
    }

    pub fn constante_float(&mut self, valor: f64) -> Result<usize, String> {
        let clave = format!("{:?}", valor); // {:?} siempre pone el punto: 3.0 → "3.0" (no choca con el int 3)
        Self::buscar_o_crear(&mut self.constantes, &mut self.cte_float, clave)
    }

    pub fn constante_string(&mut self, texto: &str) -> Result<usize, String> {
        let clave = format!("\"{}\"", texto); // con comillas: "5" (letrero) no choca con 5 (int)
        Self::buscar_o_crear(&mut self.constantes, &mut self.cte_string, clave)
    }

    fn buscar_o_crear(
        tabla: &mut HashMap<String, usize>,
        segmento: &mut Segmento,
        clave: String,
    ) -> Result<usize, String> {
        if let Some(direccion) = tabla.get(&clave) {
            return Ok(*direccion);
        }
        let direccion = segmento.pedir()?;
        tabla.insert(clave, direccion);
        Ok(direccion)
    }

    // Cuántas locales y temporales lleva usadas la función actual.
    pub fn recursos(&self) -> Recursos {
        Recursos {
            locales_int: self.local_int.usadas(),
            locales_float: self.local_float.usadas(),
            temporales_int: self.temp_int.usadas(),
            temporales_float: self.temp_float.usadas(),
            temporales_bool: self.temp_bool.usadas(),
        }
    }

    // Al empezar una función (o main) las locales y temporales vuelven a empezar,
    // porque cada función tendrá su propia memoria cuando se ejecute.
    pub fn reiniciar_locales(&mut self) {
        self.local_int.reiniciar();
        self.local_float.reiniciar();
        self.temp_int.reiniciar();
        self.temp_float.reiniciar();
        self.temp_bool.reiniciar();
    }
}
