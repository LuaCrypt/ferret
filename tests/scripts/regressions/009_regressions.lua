-- expect: pass
-- category: regressions
-- protected-string: REGRESSION_SECRET_009

local function outer(a)
    local secret = 'REGRESSION_SECRET_009'
    return function(b)
        return #secret + a + b
    end
end
local fn = outer(9)
print('regressions_009', fn(10))
