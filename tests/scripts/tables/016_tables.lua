-- expect: pass
-- category: tables

local key = 'k' .. '16'
local t = { 16, 17, named = 18, [key] = 19 }
t[4] = t[1] + t[2]
print('tables_016', t[1], t.named, t[key], t[4])
