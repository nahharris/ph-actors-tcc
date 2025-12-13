use tokio::sync::{mpsc, oneshot};
use crate::error::{AppError, FatalActorError};

pub struct App {
    tx: mpsc::Sender<Message>,
}

impl App {
    pub fn spawn(value: i32) -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new(value);
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });

        Self { tx }
    }

    pub async fn thing(&self) -> Result<i32, AppError> {
        let (tx, rx) = oneshot::channel();
        let message = Message::GetThing { tx };
        
        self.tx.send(message).await.map_err(|_| AppError::Fatal(
            FatalActorError::ActorSendFailed {
                actor_name: "App",
                operation: "get thing".to_string(),
            }
        ))?;

        rx.await.map_err(|e| AppError::Fatal(
            FatalActorError::ActorRecvFailed {
                actor_name: "App",
                operation: "get thing".to_string(),
                source: e,
            }
        ))?
    }
    
    pub async fn set_thing(&self, value: i32) -> Result<(), AppError> {
        let message = Message::SetThing { value };
        
        self.tx.send(message).await.map_err(|_| AppError::Fatal(
            FatalActorError::ActorSendFailed {
                actor_name: "App",
                operation: "set thing".to_string(),
            }
        ))
    }
}
