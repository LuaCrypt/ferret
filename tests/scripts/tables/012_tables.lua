-- expect: pass
-- category: tables

local key = 'k' .. '12'
local t = { 12, 13, named = 14, [key] = 15 }
t[4] = t[1] + t[2]
print('tables_012', t[1], t.named, t[key], t[4])
