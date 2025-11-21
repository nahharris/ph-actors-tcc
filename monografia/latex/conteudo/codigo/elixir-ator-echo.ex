defmodule Echo do
    def init do
        # Aguarda por uma mensagem
        receive do
            # Obtém o remetente e conteúdo da mensagem
            {sender, msg} -> 
                # Ecoa a resposta com sucesso (`:ok`)
                send(sender, {:ok, "Echoing: #{msg}"})
                # Loop
                init()
        end
    end
end

defmodule Example do
    def main do
        # Inicia o ator
        addr = spawn(Echo, :init, []) 
        # Envia uma mensagem
        send(addr, {self(), "Hello world"}) 
        
        # Aguarda a resposta
        receive do 
            {:ok, response} -> 
                # Exibe a resposta
                IO.puts(response) 
        end
    end
end

