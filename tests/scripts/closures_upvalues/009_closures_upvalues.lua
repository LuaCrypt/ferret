-- expect: pass
-- category: closures_upvalues

local function make_counter(seed)
    local value = seed
    return function(step)
        value = value + step
        return value
    end
end
local c = make_counter(9)
print('closures_009', c(2), c(3), c(4))
