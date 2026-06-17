local function many()
    return 2, nil, 4
end

local function outer()
    return "head", many()
end

local a, b, c, d = outer()
print("conf_return_tail", a, b, c == nil, d)
