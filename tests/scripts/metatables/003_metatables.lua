-- expect: pass
-- category: metatables

local target = {}
local t = setmetatable({}, { __newindex = function(_, key, value) target[key] = value * 2 end })
t.answer = 21
print('metatables_003', target.answer)
