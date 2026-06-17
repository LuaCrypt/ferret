-- expect: pass
-- category: tables

local key = 'k' .. '9'
local t = { 9, 10, named = 11, [key] = 12 }
t[4] = t[1] + t[2]
print('tables_009', t[1], t.named, t[key], t[4])
