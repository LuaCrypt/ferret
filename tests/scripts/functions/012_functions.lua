-- expect: pass
-- category: functions

local t = { base = 12 }
function t:add(v)
    return self.base + v
end
print('functions_012', t:add(17))
