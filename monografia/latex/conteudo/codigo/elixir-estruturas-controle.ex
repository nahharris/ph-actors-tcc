response = {:ok, "Content"}

case response do
	{:ok, content} -> IO.puts "Success with content: #{content}"
	{:err, cause} -> IO.puts "Ugh, it failed due to: #{cause}"
end

