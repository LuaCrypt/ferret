-- expect: pass
-- category: tables

local key = 'k' .. '18'
local t = { 18, 19, named = 20, [key] = 21 }
t[4] = t[1] + t[2]
print('tables_018', t[1], t.named, t[key], t[4])
