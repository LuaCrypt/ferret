-- expect: pass
-- category: tables

local key = 'k' .. '6'
local t = { 6, 7, named = 8, [key] = 9 }
t[4] = t[1] + t[2]
print('tables_006', t[1], t.named, t[key], t[4])
