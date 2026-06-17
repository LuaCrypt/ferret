-- expect: pass
-- category: functions

local f = function(a, b)
    return a * 10 + b
end
print('functions_008', f(8, 9))
