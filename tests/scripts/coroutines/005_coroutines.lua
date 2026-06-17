-- expect: pass
-- category: coroutines

local co = coroutine.create(function()
    coroutine.yield(5)
    return 6
end)
local a, b = coroutine.resume(co)
local c, d = coroutine.resume(co)
print('coroutines_005', a, b, c, d)
