-- expect: pass
-- category: functions

local f = function(a, b)
    return a * 10 + b
end
print('functions_011', f(11, 12))
