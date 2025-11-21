response = {:ok, "Some message"}
status = elem(response, 0) # :ok
response = put_elem(response, 1, "Another message") # {:ok, "Another message"}

data = [1, 2, 3, 4]
data = ["0" | data] # ["0", 1, 2, 3, 4]
data = data ++ [5] # ["0", 1, 2, 3, 4, 5]

