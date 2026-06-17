local total = 0
local a = 1
local b = 2

for i = 1, 200000 do
  a = (a + i) % 1000003
  b = (b * 3 + a) % 1000033
  if a < b then
    total = total + a
  else
    total = total + b
  end
end

print("runtime_overhead", total, a, b)
