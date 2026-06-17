local function many()
    return "a", nil, "c"
end

local t = { "root", many() }
print("conf_table_tail", t[1], t[2], t[3] == nil, t[4])
