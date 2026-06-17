-- expect: pass
-- category: metatables

local mt = { __concat = function(a, b) return a.v .. ':' .. b.v end }
local a = setmetatable({ v = 'left' }, mt)
local b = setmetatable({ v = 'right' }, mt)
print('metatables_006', a .. b)
