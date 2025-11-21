fn main() {
    let numbers = vec![1, 2, 3, 4];
    let nref = &numbers;
    println!("{}", nref[1]); // -> 2
    
    // Permitido: múltiplas referências imutáveis
    let nref2 = &numbers;
    
    // Proibido: não pode ter referência mutável 
    // quando já existem referências imutáveis
    // let mref = &mut numbers;
}

