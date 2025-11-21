use tokio;

#[tokio::main]
async fn main() {
    // Gera a primeira task
    let task1 = tokio::spawn(async {
        println!("Task 1 executando!");
    });
    
    // Gera a segunda task
    let task2 = tokio::spawn(async {
        println!("Task 2 executando!");
    });
    
    // Aguarda ambas as tasks terminarem
    let _ = tokio::join!(task1, task2);
}

