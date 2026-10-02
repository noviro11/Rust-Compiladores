use tarea::stack::Stack;

fn main() {
    let mut s = Stack::new();
    s.push(5);
    println!("{:?}", s.pop());
}
