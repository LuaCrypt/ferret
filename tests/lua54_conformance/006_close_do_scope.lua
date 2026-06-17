local closed = 0
do
    local guard = setmetatable({}, { __close = function() closed = closed + 1 end })
    local x <close> = guard
    print("conf_close_inside", closed)
end
print("conf_close_after", closed)
