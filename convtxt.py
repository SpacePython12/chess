text = input("Insert text:")
words = text.split(", ")
nums = [hex(int(word)) for word in words]
print("[" + ", ".join(nums) + "]")