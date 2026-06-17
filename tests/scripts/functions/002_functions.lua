-- expect: pass
-- category: functions

local f = function(a, b)
    return a * 10 + b
end
print('functions_002', f(2, 3))
