def loop do
    receive do
        {:set_thing, value} ->
        IO.puts("Definindo valor: #{value}")
        loop()
        {:get_thing, sender} ->
        send(sender, {:ok, 42})
        loop()
    end
end

