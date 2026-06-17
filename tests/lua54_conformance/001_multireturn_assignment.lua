local function many()
    return 1, nil, 3
end

local a, b, c = many()
print("conf_multi_assign", a, b == nil, c)
