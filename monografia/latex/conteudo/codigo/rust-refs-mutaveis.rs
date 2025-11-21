fn main() {
    let mut numbers = vec![1, 2, 3];
    let nref = &mut numbers;
    
    // Permitido: operação mutável usando &mut
    nref.push(4);
    
    // Proibido: só pode haver uma referência mutável
    // let nref2 = &mut numbers;
    
    // Proibido: não pode usar o valor original
    // enquanto existe referência mutável
    // numbers.push(5);
}

