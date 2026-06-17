local function many()
    return 7, nil, 9
end

local ok, a, b, c = pcall(many)
print("conf_pcall_multi", ok, a, b == nil, c)
