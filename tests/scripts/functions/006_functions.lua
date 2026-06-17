-- expect: pass
-- category: functions

local t = { base = 6 }
function t:add(v)
    return self.base + v
end
print('functions_006', t:add(11))
