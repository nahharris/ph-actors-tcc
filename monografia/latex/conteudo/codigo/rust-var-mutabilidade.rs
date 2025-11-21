fn main() {
    let a = 10;
    let mut b = 20;
    
    // a += 1; // Proibido: a não é mutável
    b += 1; // Permitido: b é mutável
    
    // Quando o código termina aqui, a e b são liberados da memória
}

