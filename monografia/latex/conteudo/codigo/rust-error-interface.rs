impl Env {
    pub async fn env(&self, key: ArcOsStr) -> Result<ArcStr, EnvError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        // Erro ao enviar mensagem -> Erro fatal
        self.tx
            .send(Message::Get { tx, key })
            .await
            .map_err(|_e| {
                EnvError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: "Env",
                    operation: "get environment variable".to_string(),
                })
            })?;
        
        // Erro ao receber resposta -> Erro fatal ou erro de domínio
        rx.await
            .map_err(|e| {
                EnvError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: "Env",
                    operation: "get environment variable".to_string(),
                    source: e,
                })
            })?
    }
}

