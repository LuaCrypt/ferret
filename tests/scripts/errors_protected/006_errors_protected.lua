-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    error('err_006')
end)
print('errors_006', ok)
