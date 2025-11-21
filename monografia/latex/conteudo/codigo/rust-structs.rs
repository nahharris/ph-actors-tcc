struct Point {
    timestamp: i32,
    value: f32
}

impl Point {
    // Método que pode mutar o valor de self
    fn add(&mut self, value: f32) {
        self.value += value;
    }
    
    // Método estático que serve como construtor
    fn new(timestamp: i32, value: f32) -> Self {
        Self { timestamp, value } // Sintaxe abreviada
    }
}

fn main() {
    let mut p = Point::new(20, 2.14);
    p.add(1.0);
}

