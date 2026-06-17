-- expect: pass
-- category: tables

local key = 'k' .. '7'
local t = { 7, 8, named = 9, [key] = 10 }
t[4] = t[1] + t[2]
print('tables_007', t[1], t.named, t[key], t[4])
