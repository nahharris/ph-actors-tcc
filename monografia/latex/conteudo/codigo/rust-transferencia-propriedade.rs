fn main() {
    let data = [1, 2, 3, 4]; // Cria variável imutável
    // As duas operações a seguir são proibidas pelo compilador
    // data = [2, 3, 4, 5];
    // data[0] = 0;

    let mut data2 = data; // Variável mutável data2
    // Uso de data proibido pelo compilador desse ponto em diante,
    // já que seu conteúdo foi transferido para data2
    // println!("{}", data[0]); // Erro de compilação

    // Permitido
    data2[0] = 0;
    data2 = [2, 3, 4, 5]; 
}

