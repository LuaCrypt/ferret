-- expect: pass
-- category: tables

local key = 'k' .. '2'
local t = { 2, 3, named = 4, [key] = 5 }
t[4] = t[1] + t[2]
print('tables_002', t[1], t.named, t[key], t[4])
