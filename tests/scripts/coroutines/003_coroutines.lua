-- expect: pass
-- category: coroutines

local co = coroutine.create(function()
    coroutine.yield(3)
    return 4
end)
local a, b = coroutine.resume(co)
local c, d = coroutine.resume(co)
print('coroutines_003', a, b, c, d)
