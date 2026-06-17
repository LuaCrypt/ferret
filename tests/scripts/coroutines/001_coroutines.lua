-- expect: pass
-- category: coroutines

local co = coroutine.create(function()
    coroutine.yield(1)
    return 2
end)
local a, b = coroutine.resume(co)
local c, d = coroutine.resume(co)
print('coroutines_001', a, b, c, d)
