-- expect: pass
-- category: stdlib_safe

local t = { 11, 12, 13 }
local first = table.unpack(t)
print('stdlib_011', first)
