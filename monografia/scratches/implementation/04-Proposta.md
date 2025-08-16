# Modelo Proposto

Um dos produtos desse trabalho é a proposição de um modelo para a implementação de aplicações centralizadas usando modelo de atores. Vamos aqui destrinchar a proposta.

## Os Atores

A primeira parte da proposta é tentar identificar os atores que serão necessários para implementar a aplicação. Cada ator deve ser responsável por apenas uma coisa. Em especial, deve-se tentar agrupar operações que envolvem efeitos colaterais em atores. Por exemplo:

- Gerenciar o estado da aplicação
- Lidar com eventos de entrada
- Cuidar da renderização
- Comunicação em rede

> Uma operação com efeito colateral é qualquer operação que dependa de estado que foge do escopo da aplicação: sistema de arquivos, variáveis de ambiente, entrada e saída, internet, etc. 

## Imitações de Atores

Visando a testabilidade da aplicação, atores que executem diretamente operações com efeitos colaterais devem fornecer imitações puras (ou seja, sem efeitos colaterais). As imitações devem ser tais que o invocador não saiba diferenciar quando está usando um ator real ou sua imitação. Assim, é permitida a testagem unitária de outros atores. Por exemplo: um ator que lide com variáveis de ambiente pode fornecer uma imitação que use um dicionário.

## Design de um Ator

## Passagem de Mensagens e Compartilhamento de Memória

Diferentemente do modelo de atores tradicional onde as mensagens podem ser passadas entre programas sendo executados em máquinas diferentes, aqui temos que nos preocupar apenas com comunicação entre threads. Portanto, pode-se usar canais para