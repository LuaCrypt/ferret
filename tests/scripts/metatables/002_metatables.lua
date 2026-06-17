-- expect: pass
-- category: metatables

local mt = { __index = function(_, key) if key == 'missing' then return 44 end end }
local t = setmetatable({}, mt)
print('metatables_002', t.missing)
