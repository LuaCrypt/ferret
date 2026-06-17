-- expect: pass
-- category: tables

local key = 'k' .. '1'
local t = { 1, 2, named = 3, [key] = 4 }
t[4] = t[1] + t[2]
print('tables_001', t[1], t.named, t[key], t[4])
