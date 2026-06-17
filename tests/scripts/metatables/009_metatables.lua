-- expect: pass
-- category: metatables

local mt = { __eq = function(a, b) return a.v == b.v end }
local a = setmetatable({ v = 8 }, mt)
local b = setmetatable({ v = 8 }, mt)
print('metatables_009', a == b)
