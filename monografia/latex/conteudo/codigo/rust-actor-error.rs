#[derive(Debug, Error, Diagnostic)]
pub enum EnvError {
    /// Variável de ambiente não encontrada
    NotFound {
        name: String,
    },
    
    /// Erro fatal ocorrido durante operações do ator
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

