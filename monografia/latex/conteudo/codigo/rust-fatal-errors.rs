use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("Fatal actor error")]
pub enum FatalActorError {
    /// Falha ao enviar mensagem para o ator
    #[error("Failed to send message to actor '{actor_name}' for operation '{operation}'")]
    #[diagnostic(
        code(fatal::actor_send_failed),
        help("The message could not be sent to the actor. The actor may have died or the channel may be closed.")
    )]
    ActorSendFailed {
        actor_name: &'static str,
        operation: String,
    },
    
    /// Falha ao receber resposta do ator
    #[error("Failed to receive response from actor '{actor_name}' for operation '{operation}'")]
    #[diagnostic(
        code(fatal::actor_recv_failed),
        help("The response could not be received from the actor. The actor may have died or the channel may be closed.")
    )]
    ActorRecvFailed {
        actor_name: &'static str,
        operation: String,
        #[source]
        source: tokio::sync::oneshot::error::RecvError,
    },
}

