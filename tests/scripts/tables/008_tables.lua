-- expect: pass
-- category: tables

local key = 'k' .. '8'
local t = { 8, 9, named = 10, [key] = 11 }
t[4] = t[1] + t[2]
print('tables_008', t[1], t.named, t[key], t[4])
