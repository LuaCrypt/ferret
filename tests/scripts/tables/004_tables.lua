-- expect: pass
-- category: tables

local key = 'k' .. '4'
local t = { 4, 5, named = 6, [key] = 7 }
t[4] = t[1] + t[2]
print('tables_004', t[1], t.named, t[key], t[4])
