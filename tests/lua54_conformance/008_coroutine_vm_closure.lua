local co = coroutine.create(function()
    coroutine.yield("yielded")
    return "done"
end)

local a, b = coroutine.resume(co)
local c, d = coroutine.resume(co)
print("conf_coroutine", a, b, c, d)
