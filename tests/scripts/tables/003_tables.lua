-- expect: pass
-- category: tables

local key = 'k' .. '3'
local t = { 3, 4, named = 5, [key] = 6 }
t[4] = t[1] + t[2]
print('tables_003', t[1], t.named, t[key], t[4])
