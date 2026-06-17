-- expect: pass
-- category: metatables

local mt = { __call = function(_, a, b) return a + b end }
local t = setmetatable({}, mt)
print('metatables_005', t(10, 12))
