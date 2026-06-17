-- expect: pass
-- category: tables

local key = 'k' .. '17'
local t = { 17, 18, named = 19, [key] = 20 }
t[4] = t[1] + t[2]
print('tables_017', t[1], t.named, t[key], t[4])
