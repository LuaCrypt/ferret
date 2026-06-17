-- expect: pass
-- category: functions

local t = { base = 15 }
function t:add(v)
    return self.base + v
end
print('functions_015', t:add(20))
