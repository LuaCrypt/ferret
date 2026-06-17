local function echo(...)
    return "head", ...
end

local a, b, c = echo(8, nil)
print("conf_vararg_tail", a, b, c == nil)
