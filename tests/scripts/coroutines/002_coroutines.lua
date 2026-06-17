-- expect: pass
-- category: coroutines

local co = coroutine.create(function()
    coroutine.yield(2)
    return 3
end)
local a, b = coroutine.resume(co)
local c, d = coroutine.resume(co)
print('coroutines_002', a, b, c, d)
