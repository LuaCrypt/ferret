-- expect: pass
-- category: tables

local key = 'k' .. '13'
local t = { 13, 14, named = 15, [key] = 16 }
t[4] = t[1] + t[2]
print('tables_013', t[1], t.named, t[key], t[4])
