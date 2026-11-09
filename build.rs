// Se ejecuta ANTES de compilar el proyecto: busca archivos .lalrpop en src/
// y genera el código Rust del parser (queda dentro de target/).
fn main() {
    lalrpop::process_src().unwrap();
}
