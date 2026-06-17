-- expect: pass
-- category: stdlib_safe

local t = { 3, 4, 5 }
local first = table.unpack(t)
print('stdlib_003', first)
