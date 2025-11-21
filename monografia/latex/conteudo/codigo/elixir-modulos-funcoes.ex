defmodule Math do
  # Função pública
  def add(a, b) do
    a + b
  end

  # Forma abreviada de uma linha
  def mul(a, b), do: a * b

  # Função privada (apenas chamável dentro do módulo)
  defp square(x), do: x * x

  # Múltiplas declarações de função com casamento de padrões
  def abs(n) when is_number(n) and n < 0, do: -n
  def abs(n) when is_number(n), do: n
end

IO.puts(Math.add(2, 3))   # 5
IO.puts(Math.mul(4, 5))   # 20

