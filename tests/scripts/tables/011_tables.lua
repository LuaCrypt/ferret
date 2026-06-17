-- expect: pass
-- category: tables

local key = 'k' .. '11'
local t = { 11, 12, named = 13, [key] = 14 }
t[4] = t[1] + t[2]
print('tables_011', t[1], t.named, t[key], t[4])
