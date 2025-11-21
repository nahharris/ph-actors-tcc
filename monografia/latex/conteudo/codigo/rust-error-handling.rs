impl Core {
    pub async fn init(self, mut rx: mpsc::Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            match msg {
                Message::Get { tx, key } => {
                    // Processa a operação e pode retornar um erro
                    let result = self.handle_get_env(key);
                    let _ = tx.send(result);
                }
                // ...
            }
        }
    }
    
    fn handle_get_env(&self, key: ArcOsStr) -> Result<ArcStr, EnvError> {
        std::env::var(key.as_ref())
            .map(|s| ArcStr::from(&s))
            .map_err(|_| EnvError::NotFound {
                name: key.to_string_lossy().to_string(),
            })
    }
}

