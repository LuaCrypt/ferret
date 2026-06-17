-- expect: pass
-- category: coroutines

local co = coroutine.create(function()
    coroutine.yield(4)
    return 5
end)
local a, b = coroutine.resume(co)
local c, d = coroutine.resume(co)
print('coroutines_004', a, b, c, d)
