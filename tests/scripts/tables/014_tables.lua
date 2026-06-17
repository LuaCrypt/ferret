-- expect: pass
-- category: tables

local key = 'k' .. '14'
local t = { 14, 15, named = 16, [key] = 17 }
t[4] = t[1] + t[2]
print('tables_014', t[1], t.named, t[key], t[4])
