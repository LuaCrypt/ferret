-- expect: pass
-- category: stdlib_safe

local t = { 7, 8, 9 }
local first = table.unpack(t)
print('stdlib_007', first)
