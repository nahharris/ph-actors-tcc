fn main() {
    let numbers = vec![1, 2, 3, 4];
    let nref = &numbers;
    println!("{}", nref[1]); // -> 2
    
    drop(numbers); // Libera numbers da memória
    
    // A partir deste ponto, o compilador proibirá
    // o uso de tanto numbers quanto nref
    // println!("{}", nref[1]); // Erro de compilação
}

