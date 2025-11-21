// Loop que processa mensagens até o canal ser fechado
while let Some(message) = rx.recv().await {
    match message {
        Message::SetThing { value } => {
            println!("Definindo valor: {}", value);
        }
        Message::GetThing { tx } => {
            let _ = tx.send(42); // Envia resposta
        }
    }
}
// Quando não há mais Senders, rx.recv() retorna None
// e o loop se encerra automaticamente
println!("Ator finalizando - canal fechado");

