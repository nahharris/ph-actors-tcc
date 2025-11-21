x = 1
1 = x # Isso funciona
2 = x # Isso irá causar um erro

# Desestruturar tuplas
{status, message} = {:err, "the system panicked"}

IO.puts "It's a #{status}. Because #{message}"
# It's a err. Because the system panicked

